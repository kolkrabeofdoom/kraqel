//! Explicit, Windows-only research feature. Normal application startup never calls this module.
use kraqel_isolation_test_adapter::{DurableStore, ImportReceipt, TaskControl, TaskError};
use kraqel_isolation_transport::{
    CrashPoint, LiveSession, ReceivedGrant, RestartEventKind, receive_from_launcher,
    receive_live_session, receive_restart,
};
mod restart;
use kraqel_storage::project_manager::ProjectManager;
use restart::RestartControl;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    document_count: u32,
    asset_count: u32,
    project_id: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    phase: &'static str,
    error: Option<TaskError>,
    receipt: Option<Receipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovered: Option<bool>,
}
struct Inner {
    grant: Mutex<Option<Source>>,
    cancel: AtomicBool,
    release: AtomicBool,
    phase: AtomicU8,
    result: Mutex<(Option<TaskError>, Option<Receipt>)>,
    data: PathBuf,
    exit_requested: AtomicBool,
    renderer_seen: AtomicBool,
    focus_probe: Mutex<Vec<FocusSample>>,
    store: Mutex<DurableStore>,
    restart: Option<Arc<RestartControl>>,
}

impl Inner {
    fn claim(&self, store: &mut DurableStore, id: uuid::Uuid) -> Result<(), TaskError> {
        store.claim(id).inspect_err(|error| {
            if let Some(restart) = &self.restart
                && *error == TaskError::AlreadyConsumed
            {
                restart.event(RestartEventKind::ReuseRejected, None);
            }
        })
    }
    fn control(&self) -> TaskControl {
        TaskControl {
            now_unix_ms: now(),
            cancelled: self.cancel.load(Ordering::SeqCst),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FocusEvent {
    Action,
    Phase,
    Focus,
    Blur,
    Navigation,
    Pointer,
    Handoff,
    AfterHandoff,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FocusPhase {
    Unknown,
    Ready,
    Running,
    Awaiting,
    Done,
    Cancelled,
    Failed,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FocusTarget {
    Body,
    Import,
    Continue,
    Cancel,
    Replay,
    Other,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FocusSample {
    event: FocusEvent,
    phase: FocusPhase,
    focused: FocusTarget,
    has_focus: bool,
    preserve_user_focus: bool,
    trusted_input: bool,
}
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn isolation_test_focus_probe(
    window: WebviewWindow,
    state: State<'_, UiState>,
    sample: FocusSample,
) -> Result<(), TaskError> {
    guard(&window)?;
    let mut records = state
        .0
        .focus_probe
        .lock()
        .map_err(|_| TaskError::ImportFailed)?;
    if records.len() >= 64 {
        return Err(TaskError::InvalidTask);
    }
    records.push(sample);
    let bytes = serde_json::to_vec(&*records).map_err(|_| TaskError::ImportFailed)?;
    if bytes.len() > 16384 {
        return Err(TaskError::InvalidTask);
    }
    fs::write(state.0.data.join("ui-focus.json"), bytes).map_err(|_| TaskError::ImportFailed)
}
enum Source {
    Prepared(ReceivedGrant),
    Live(LiveSession),
    Synthetic(uuid::Uuid),
}
impl Source {
    fn task_id(&self) -> uuid::Uuid {
        match self {
            Self::Prepared(grant) => grant.task.task_id,
            Self::Live(session) => session.task_id(),
            Self::Synthetic(id) => *id,
        }
    }
}
#[derive(Clone)]
struct UiState(Arc<Inner>);
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}
fn allowed_origin(label: &str, url: &tauri::Url) -> bool {
    label == "isolation-test"
        && url.path() == "/isolation-test.html"
        && matches!(
            (url.scheme(), url.host_str()),
            ("http", Some("tauri.localhost")) | ("tauri", Some("localhost"))
        )
}
fn guard(window: &WebviewWindow) -> Result<(), TaskError> {
    if window
        .url()
        .is_ok_and(|url| allowed_origin(window.label(), &url))
    {
        Ok(())
    } else {
        Err(TaskError::InvalidTask)
    }
}
fn view(inner: &Inner) -> Result<View, TaskError> {
    let (error, receipt) = inner
        .result
        .lock()
        .map_err(|_| TaskError::ImportFailed)?
        .clone();
    let phase = match inner.phase.load(Ordering::SeqCst) {
        0 => "ready",
        1 => "running",
        2 => "awaiting",
        3 => "done",
        4 => "cancelled",
        _ => "failed",
    };
    Ok(View {
        phase,
        error,
        receipt,
        recovered: inner
            .restart
            .as_ref()
            .filter(|r| r.ticket.recovery)
            .map(|_| true),
    })
}
fn finish(inner: &Inner, result: Result<ImportReceipt, TaskError>) {
    let (error, receipt, phase) = match result {
        Ok(r) => (
            None,
            Some(Receipt {
                document_count: r.document_count,
                asset_count: r.asset_count,
                project_id: r.project_id.to_string(),
            }),
            3,
        ),
        Err(e) => (Some(e), None, if e == TaskError::Cancelled { 4 } else { 5 }),
    };
    if let Ok(mut stored) = inner.result.lock() {
        *stored = (error, receipt);
    }
    inner.phase.store(phase, Ordering::SeqCst);
    if let Ok(state) =
        view(inner).and_then(|s| serde_json::to_vec(&s).map_err(|_| TaskError::ImportFailed))
    {
        let _ = fs::write(inner.data.join("ui-result.json"), state);
    }
}
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri injects owned window/state command arguments.
fn isolation_test_status(
    window: WebviewWindow,
    state: State<'_, UiState>,
) -> Result<View, TaskError> {
    guard(&window)?;
    if !state.0.renderer_seen.swap(true, Ordering::SeqCst) {
        if let Some(restart) = &state.0.restart {
            restart.event(RestartEventKind::Renderer, None);
        }
        let _ = fs::write(
            state.0.data.join("ui-renderer.json"),
            b"{\"nativeStatusInvoked\":true}",
        );
    }
    view(&state.0)
}
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri command injection.
fn isolation_test_continue(
    window: WebviewWindow,
    state: State<'_, UiState>,
) -> Result<(), TaskError> {
    guard(&window)?;
    if state.0.phase.load(Ordering::SeqCst) != 2 {
        return Err(TaskError::InvalidTask);
    }
    state.0.release.store(true, Ordering::SeqCst);
    Ok(())
}
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri command injection.
fn isolation_test_cancel(
    window: WebviewWindow,
    state: State<'_, UiState>,
) -> Result<(), TaskError> {
    guard(&window)?;
    let mut slot = state.0.grant.lock().map_err(|_| TaskError::ImportFailed)?;
    if state.0.phase.load(Ordering::SeqCst) >= 3 {
        return Err(TaskError::AlreadyConsumed);
    }
    state.0.cancel.store(true, Ordering::SeqCst);
    state.0.release.store(true, Ordering::SeqCst);
    if let Some(grant) = slot.take() {
        let mut store = state
            .0
            .store
            .lock()
            .map_err(|_| TaskError::StateUnavailable)?;
        store.claim(grant.task_id())?;
        store.refuse(TaskError::Cancelled)?;
        drop(grant);
        finish(&state.0, Err(TaskError::Cancelled));
    }
    Ok(())
}
#[tauri::command]
async fn isolation_test_import(
    window: WebviewWindow,
    state: State<'_, UiState>,
    app: tauri::AppHandle,
) -> Result<View, TaskError> {
    guard(&window)?;
    let inner = Arc::clone(&state.0);
    let source = {
        let mut slot = inner.grant.lock().map_err(|_| TaskError::ImportFailed)?;
        let Some(grant) = slot.take() else {
            if let Some(restart) = &inner.restart {
                restart.event(RestartEventKind::Replay, None);
            }
            let _ = fs::write(
                inner.data.join("ui-replay.json"),
                b"{\"replayRejected\":true}",
            );
            return Err(TaskError::AlreadyConsumed);
        };
        inner.phase.store(1, Ordering::SeqCst);
        grant
    };
    let worker = Arc::clone(&inner);
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut store = worker
            .store
            .lock()
            .map_err(|_| TaskError::StateUnavailable)?;
        worker.claim(&mut store, source.task_id())?;
        if let Some(restart) = &worker.restart {
            restart.checkpoint(CrashPoint::Claimed);
        }
        let selected: Result<(_, Box<dyn std::io::Read + Send>), TaskError> = (|| match source {
            Source::Prepared(grant) => Ok((
                grant.task,
                Box::new(grant.input) as Box<dyn std::io::Read + Send>,
            )),
            Source::Synthetic(id) => {
                let control = Arc::clone(worker.restart.as_ref().ok_or(TaskError::InvalidTask)?);
                Ok(restart::synthetic(id, control))
            }
            Source::Live(session) => {
                let Some((task, input)) = session
                    .select(|| worker.cancel.load(Ordering::SeqCst))
                    .map_err(|_| {
                    if worker.cancel.load(Ordering::SeqCst) {
                        TaskError::Cancelled
                    } else {
                        TaskError::ReadFailed
                    }
                })?
                else {
                    return Err(TaskError::Cancelled);
                };
                Ok((task, Box::new(input) as Box<dyn std::io::Read + Send>))
            }
        })();
        let (task, mut input) = match selected {
            Ok(selected) => selected,
            Err(error) => {
                store.refuse(error)?;
                return Err(error);
            }
        };
        let receipt = store.execute(
            &task,
            &mut input,
            || worker.control(),
            || {
                worker.phase.store(2, Ordering::SeqCst);
                while !worker.release.load(Ordering::SeqCst)
                    && !worker.cancel.load(Ordering::SeqCst)
                    && now() < task.expires_unix_ms
                {
                    std::thread::sleep(Duration::from_millis(50));
                }
                worker.phase.store(1, Ordering::SeqCst);
                worker.control()
            },
            |point| {
                if let Some(restart) = &worker.restart {
                    restart.milestone(point);
                }
            },
        )?;
        if let Some(restart) = &worker.restart {
            restart.checkpoint(CrashPoint::Completed);
        }
        Ok(receipt)
    })
    .await
    .map_err(|_| TaskError::ImportFailed)
    .and_then(|result| result);
    finish(&inner, result);
    if inner.exit_requested.load(Ordering::SeqCst) {
        app.exit(0);
    }
    view(&inner)
}

type Startup = (PathBuf, Option<Source>, Option<Arc<RestartControl>>);
fn receive_source() -> Result<Startup, kraqel_isolation_transport::ChannelError> {
    if std::env::args().nth(1).as_deref() == Some("ui-restart") {
        receive_restart().map(|grant| {
            let source =
                (!grant.ticket.recovery).then_some(Source::Synthetic(grant.ticket.task_id));
            (
                grant.data_root.clone(),
                source,
                Some(Arc::new(RestartControl::new(grant))),
            )
        })
    } else if std::env::args().nth(1).as_deref() == Some("ui-live") {
        receive_live_session()
            .map(|session| (session.data_root.clone(), Some(Source::Live(session)), None))
    } else {
        receive_from_launcher()
            .map(|grant| (grant.data_root.clone(), Some(Source::Prepared(grant)), None))
    }
}
/// Runs only with a validated native grant inside the research container.
pub fn run() {
    let Ok((data, grant, restart)) = receive_source() else {
        eprintln!("Isolationstest: kein gültiger nativer Auftrag.");
        std::process::exit(2)
    };
    if fs::create_dir_all(data.join("app/projects")).is_err()
        || fs::create_dir_all(data.join("webview")).is_err()
    {
        std::process::exit(2);
    }
    let live = matches!(&grant, Some(Source::Live(_)));
    let store = ProjectManager::new(data.join("app/projects"))
        .map_err(|_| TaskError::StateUnavailable)
        .and_then(|manager| DurableStore::open(&data.join("app/tasks"), &manager));
    let Ok(mut store) = store else {
        eprintln!("Isolationstest: Auftragszustand nicht verfügbar.");
        std::process::exit(2);
    };
    let recovered = restart
        .as_ref()
        .filter(|restart| restart.ticket.recovery)
        .map(|restart| restart::recover(&mut store, &data, restart))
        .transpose();
    let Ok(recovered) = recovered else {
        std::process::exit(2);
    };
    let inner = Arc::new(Inner {
        grant: Mutex::new(grant),
        cancel: AtomicBool::new(false),
        release: AtomicBool::new(false),
        phase: AtomicU8::new(0),
        result: Mutex::new((None, None)),
        data: data.clone(),
        exit_requested: AtomicBool::new(false),
        renderer_seen: AtomicBool::new(false),
        focus_probe: Mutex::new(Vec::new()),
        store: Mutex::new(store),
        restart: restart.clone(),
    });
    if let Some(result) = recovered {
        finish(&inner, result);
    }
    let ready = if let Some(restart) = &restart {
        serde_json::json!({"restartVersion":1,"recoveryOnly":restart.ticket.recovery,"handleInheritanceCleared":true})
    } else if live {
        serde_json::json!({"channelVersion":2,"sessionBound":true,"selectionPending":true,"handleInheritanceCleared":true})
    } else {
        serde_json::json!({"channelVersion":1,"taskBound":true,"handleInheritanceCleared":true})
    };
    if let Ok(bytes) = serde_json::to_vec(&ready) {
        let _ = fs::write(data.join("ui-ready.json"), bytes);
    }
    let mut context = tauri::generate_context!();
    context.config_mut().identifier = "org.kraqel.desktop.isolation-ui-test".into();
    context.config_mut().app.windows.clear();
    let state = UiState(Arc::clone(&inner));
    let app = tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            isolation_test_status,
            isolation_test_focus_probe,
            isolation_test_import,
            isolation_test_continue,
            isolation_test_cancel
        ])
        .setup(move |app| {
            WebviewWindowBuilder::new(
                app,
                "isolation-test",
                WebviewUrl::App("isolation-test.html".into()),
            )
            .title("KRAQEL Isolierter Dateiauftrag")
            .inner_size(880., 620.)
            .data_directory(data.join("webview"))
            .on_page_load(move |window, payload| {
                if payload.event() == tauri::webview::PageLoadEvent::Finished
                    && allowed_origin("isolation-test", payload.url())
                    && let Some(restart) = &restart
                {
                    let _ = window.eval(restart::automation(restart.ticket.recovery));
                }
            })
            .build()?;
            Ok(())
        })
        .build(context);
    if let Ok(app) = app {
        app.run(move |_, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event
                && matches!(inner.phase.load(Ordering::SeqCst), 1 | 2)
            {
                api.prevent_exit();
                inner.exit_requested.store(true, Ordering::SeqCst);
                inner.cancel.store(true, Ordering::SeqCst);
                inner.release.store(true, Ordering::SeqCst);
            }
        });
    } else {
        eprintln!("Isolationstest: Fensterstart fehlgeschlagen.");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_probe_rejects_paths_free_text_and_unknown_fields() {
        let sample = serde_json::json!({"event":"phase","phase":"awaiting","focused":"cancel","hasFocus":true,"preserveUserFocus":false,"trustedInput":false});
        assert!(serde_json::from_value::<FocusSample>(sample.clone()).is_ok());
        let mut unknown = sample.clone();
        unknown["path"] = serde_json::json!("C:\\private");
        assert!(serde_json::from_value::<FocusSample>(unknown).is_err());
        let mut free_text = sample;
        free_text["focused"] = serde_json::json!("private manuscript text");
        assert!(serde_json::from_value::<FocusSample>(free_text).is_err());
    }
    #[test]
    fn only_the_bundled_research_window_can_invoke_commands() {
        assert!(allowed_origin(
            "isolation-test",
            &tauri::Url::parse("http://tauri.localhost/isolation-test.html").unwrap()
        ));
        for (label, url) in [
            ("main", "http://tauri.localhost/isolation-test.html"),
            ("isolation-test", "http://localhost/isolation-test.html"),
            ("isolation-test", "https://example.com/isolation-test.html"),
            ("isolation-test", "http://tauri.localhost/index.html"),
        ] {
            assert!(!allowed_origin(label, &tauri::Url::parse(url).unwrap()));
        }
    }
}
