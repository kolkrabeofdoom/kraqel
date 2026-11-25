//! Menu commands use the same requests and control state as the scene window.
use super::*;

pub(super) const SEARCH_MENU: usize = 217;

pub(super) fn install(hwnd: HWND, edit_mode: bool) -> bool {
    let file = [
        (SAVE as usize, "&Speichern\tStrg+S"),
        (CLOSE as usize, "S&chließen\tAlt+F4"),
    ];
    let edit = [
        (SELECT_PARAGRAPH, "&Alles auswählen\tStrg+A"),
        (SEARCH_MENU, "&Suchen …\tStrg+F"),
        (project_search::NEXT as usize, "&Nächster Treffer\tF3"),
        (
            project_search::PREVIOUS as usize,
            "&Vorheriger Treffer\tUmschalt+F3",
        ),
    ];
    if edit_mode {
        let mut commands = vec![
            (UNDO, "&Rückgängig\tStrg+Z"),
            (REDO, "&Wiederholen\tStrg+Y"),
        ];
        commands.extend(edit);
        commands.extend([
            (project_replace::OPEN as usize, "&Ersetzen …\tStrg+H"),
            (COMPARE as usize, "&Fassungen vergleichen …"),
        ]);
        install_menu_groups(
            hwnd,
            &[
                ("&Datei", &file),
                ("&Bearbeiten", &commands),
                ("&Ansicht", &[(FOCUS as usize, "&Fokusmodus\tF11")]),
            ],
        )
    } else {
        install_menu_groups(hwnd, &[("&Datei", &file[1..]), ("&Bearbeiten", &edit)])
    }
}

pub(super) unsafe fn refresh(hwnd: HWND) {
    unsafe {
        let menu = GetMenu(hwnd);
        let save = GetDlgItem(hwnd, SAVE);
        if menu.is_null() || save.is_null() {
            return;
        }
        let editable = GetWindowLongW(GetDlgItem(hwnd, EDIT), GWL_STYLE) & ES_READONLY == 0;
        let compare = GetDlgItem(hwnd, COMPARE);
        enable_menu_command(hwnd, UNDO, editable && project_history::available(false));
        enable_menu_command(hwnd, REDO, editable && project_history::available(true));
        enable_menu_command(hwnd, SAVE as usize, editable && IsWindowEnabled(save) != 0);
        enable_menu_command(hwnd, CLOSE as usize, editable);
        enable_menu_command(hwnd, project_replace::OPEN as usize, editable);
        enable_menu_command(
            hwnd,
            COMPARE as usize,
            editable && IsWindowVisible(compare) != 0 && IsWindowEnabled(compare) != 0,
        );
        CheckMenuItem(
            menu,
            FOCUS as u32,
            MF_BYCOMMAND
                | if focus_mode(hwnd) {
                    MF_CHECKED
                } else {
                    MF_UNCHECKED
                },
        );
    }
}
