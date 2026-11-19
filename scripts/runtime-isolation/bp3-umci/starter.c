#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <userenv.h>
#include <aclapi.h>
#include <strsafe.h>
#include <stdint.h>
#include <stdio.h>

typedef struct {
    HANDLE job;
    HANDLE accounting_job;
    PSID sid;
    const wchar_t *worker;
    const wchar_t *directory;
} probe;

static BOOL expected_token(HANDLE process, PSID sid) {
    HANDLE token;
    DWORD container = 0, bytes = 0;
    union { TOKEN_APPCONTAINER_INFORMATION info; BYTE bytes[256]; } owner;
    TOKEN_GROUPS caps = {0};
    if (!OpenProcessToken(process, TOKEN_QUERY, &token))
        return FALSE;
    BOOL ok = GetTokenInformation(token, TokenIsAppContainer, &container,
                                  sizeof(container), &bytes) && container &&
        GetTokenInformation(token, TokenCapabilities, &caps, sizeof(caps), &bytes) &&
        caps.GroupCount == 0 &&
        GetTokenInformation(token, TokenAppContainerSid, &owner, sizeof(owner), &bytes) &&
        owner.info.TokenAppContainer && EqualSid(sid, owner.info.TokenAppContainer);
    CloseHandle(token);
    return ok;
}

static DWORD allow_read(const wchar_t *path, PSID sid) {
    PACL old = NULL, acl = NULL;
    PSECURITY_DESCRIPTOR descriptor = NULL;
    DWORD error = GetNamedSecurityInfoW((LPWSTR)path, SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
                                        NULL, NULL, &old, NULL, &descriptor);
    if (error != ERROR_SUCCESS)
        return error;
    EXPLICIT_ACCESSW entry = {0};
    entry.grfAccessPermissions = FILE_GENERIC_READ | FILE_GENERIC_EXECUTE;
    entry.grfAccessMode = GRANT_ACCESS;
    entry.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    entry.Trustee.ptstrName = (LPWSTR)sid;
    error = SetEntriesInAclW(1, &entry, old, &acl);
    if (!error)
        error = SetNamedSecurityInfoW((LPWSTR)path, SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
                                     NULL, NULL, acl, NULL);
    LocalFree(acl);
    LocalFree(descriptor);
    return error;
}

typedef enum { LEAF, STATIC_ALLOWED, STATIC_FOREIGN, CRASH_LEAF } role_kind;

static BOOL role(const probe *test, HANDLE input, role_kind kind) {
    SECURITY_CAPABILITIES caps = { test->sid, NULL, 0, 0 };
    DWORD child = PROCESS_CREATION_CHILD_PROCESS_RESTRICTED;
    DWORD64 code = PROCESS_CREATION_MITIGATION_POLICY_PROHIBIT_DYNAMIC_CODE_ALWAYS_ON;
    HANDLE jobs[] = { test->accounting_job, test->job };
    BOOL delegate = input != NULL;
    const char *label = kind == STATIC_ALLOWED ? "static-allowed" :
                        kind == STATIC_FOREIGN ? "static-foreign" : delegate ? "reader" : "empty";
    SIZE_T bytes = 0;
    InitializeProcThreadAttributeList(NULL, delegate ? 5 : 4, 0, &bytes);
    LPPROC_THREAD_ATTRIBUTE_LIST attrs = bytes ? HeapAlloc(GetProcessHeap(), 0, bytes) : NULL;
    if (!attrs)
        return FALSE;
    BOOL initialized = InitializeProcThreadAttributeList(attrs, delegate ? 5 : 4, 0, &bytes);
    BOOL ok = FALSE;
    PROCESS_INFORMATION p = {0};
    DWORD stage = 1;
    if (!initialized)
        goto done;
    if (!UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
                                   &caps, sizeof(caps), NULL, NULL) ||
        !UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_JOB_LIST,
                                   test->accounting_job ? jobs : &jobs[1],
                                   test->accounting_job ? sizeof(jobs) : sizeof(HANDLE), NULL, NULL) ||
        !UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY,
                                   &child, sizeof(child), NULL, NULL) ||
        !UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY,
                                   &code, sizeof(code), NULL, NULL) ||
        (delegate && !UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                                               &input, sizeof(input), NULL, NULL)))
        goto done;
    wchar_t command[64] = L"bp3-worker none";
    if (delegate && FAILED(StringCchPrintfW(command, 64, kind == CRASH_LEAF ? L"bp3-worker crash-ready-%llu" : L"bp3-worker %llu",
                                           (unsigned long long)(uintptr_t)input)))
        goto done;
    STARTUPINFOEXW si = {0};
    si.StartupInfo.cb = sizeof(si);
    si.lpAttributeList = attrs;
    stage = 2;
    if (!CreateProcessW(test->worker, command, NULL, NULL, delegate,
                        CREATE_SUSPENDED | CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
                        NULL, test->directory, &si.StartupInfo, &p))
        goto done;
    stage = 3;
    BOOL member = FALSE;
    FILETIME created, exited, kernel, user;
    wchar_t actual_image[MAX_PATH];
    DWORD image_chars = MAX_PATH;
    PROCESS_MITIGATION_DYNAMIC_CODE_POLICY dynamic;
    PROCESS_MITIGATION_CHILD_PROCESS_POLICY policy;
    if (GetProcessId(p.hProcess) != p.dwProcessId ||
        !GetProcessTimes(p.hProcess, &created, &exited, &kernel, &user) ||
        !(created.dwHighDateTime || created.dwLowDateTime) ||
        !QueryFullProcessImageNameW(p.hProcess, 0, actual_image, &image_chars) ||
        _wcsicmp(test->worker, actual_image) != 0 ||
        !IsProcessInJob(p.hProcess, test->job, &member) || !member ||
        !expected_token(p.hProcess, test->sid) ||
        !GetProcessMitigationPolicy(p.hProcess, ProcessChildProcessPolicy,
                                    &policy, sizeof(policy)) ||
        !policy.NoChildProcessCreation || policy.AuditNoChildProcessCreation ||
        policy.AllowSecureProcessCreation ||
        !GetProcessMitigationPolicy(p.hProcess, ProcessDynamicCodePolicy,
                                    &dynamic, sizeof(dynamic)) ||
        !dynamic.ProhibitDynamicCode || dynamic.AllowThreadOptOut ||
        dynamic.AllowRemoteDowngrade || dynamic.AuditProhibitDynamicCode)
        goto done;
    stage = 4;
    if (ResumeThread(p.hThread) != 1 || WaitForSingleObject(p.hProcess, kind == CRASH_LEAF ? 45000 : 5000) != WAIT_OBJECT_0)
        goto done;
    DWORD code_out = 0;
    stage = 5;
    if (GetExitCodeProcess(p.hProcess, &code_out)) {
        if (kind == STATIC_FOREIGN)
            /* STATUS_INVALID_IMAGE_HASH or STATUS_SYSTEM_INTEGRITY_POLICY_VIOLATION */
            ok = code_out == 0xC0000428UL || code_out == 0xC0E90002UL;
        else
            ok = code_out == (kind == STATIC_ALLOWED ? 50UL : 40UL);
    }
    printf("role %s worker-exit %lu\n", label, code_out);
done:
    if (!ok)
        printf("role %s failed-stage %lu win32 %lu\n", label,
               stage, GetLastError());
    if (p.hProcess) {
        if (!ok) TerminateProcess(p.hProcess, 2);
        CloseHandle(p.hThread);
        CloseHandle(p.hProcess);
    }
    if (initialized) DeleteProcThreadAttributeList(attrs);
    HeapFree(GetProcessHeap(), 0, attrs);
    return ok;
}

/* Keep only the accounting job open in the observer. Holding the owner's
   kill-on-close job here would itself suppress the behavior under test. */
static BOOL crash_trial(const wchar_t *starter, const wchar_t *profile, const probe *test) {
    SECURITY_ATTRIBUTES inherit = { sizeof(inherit), NULL, TRUE };
    HANDLE accounting = CreateJobObjectW(&inherit, NULL);
    HANDLE ready = CreateEventW(NULL, TRUE, FALSE, NULL);
    HANDLE signal = NULL, leaf = NULL;
    PROCESS_INFORMATION owner = {0};
    BOOL ok = FALSE, initialized = FALSE;
    SIZE_T bytes = 0;
    InitializeProcThreadAttributeList(NULL, 1, 0, &bytes);
    LPPROC_THREAD_ATTRIBUTE_LIST attrs = HeapAlloc(GetProcessHeap(), 0, bytes);
    if (!accounting || !ready || !attrs ||
        !DuplicateHandle(GetCurrentProcess(), ready, GetCurrentProcess(), &signal,
                         EVENT_MODIFY_STATE, TRUE, 0))
        goto done;
    HANDLE handles[] = {accounting, signal};
    initialized = InitializeProcThreadAttributeList(attrs, 1, 0, &bytes);
    if (!initialized || !UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                                                   handles, sizeof(handles), NULL, NULL))
        goto done;
    wchar_t cmd[256];
    if (FAILED(StringCchPrintfW(cmd, 256, L"bp3-starter --crash-owner %llu %llu %s",
                               (unsigned long long)(uintptr_t)accounting,
                               (unsigned long long)(uintptr_t)signal, profile)))
        goto done;
    STARTUPINFOEXW startup = {0};
    startup.StartupInfo.cb = sizeof(startup);
    startup.lpAttributeList = attrs;
    if (!CreateProcessW(starter, cmd, NULL, NULL, TRUE, EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW,
                        NULL, test->directory, &startup.StartupInfo, &owner))
        goto done;
    HANDLE waiters[] = {owner.hProcess, ready};
    if (WaitForMultipleObjects(2, waiters, FALSE, 10000) != WAIT_OBJECT_0 + 1)
        goto done;
    JOBOBJECT_BASIC_PROCESS_ID_LIST list;
    if (!QueryInformationJobObject(accounting, JobObjectBasicProcessIdList, &list, sizeof(list), NULL) ||
        list.NumberOfAssignedProcesses != 1 || list.NumberOfProcessIdsInList != 1)
        goto done;
    leaf = OpenProcess(SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, FALSE, (DWORD)list.ProcessIdList[0]);
    BOOL member = FALSE;
    DWORD len = MAX_PATH, owner_exit, leaf_exit;
    wchar_t image[MAX_PATH];
    if (!leaf || !IsProcessInJob(leaf, accounting, &member) || !member ||
        !QueryFullProcessImageNameW(leaf, 0, image, &len) || _wcsicmp(image, test->worker) ||
        WaitForSingleObject(leaf, 0) != WAIT_TIMEOUT || WaitForSingleObject(owner.hProcess, 0) != WAIT_TIMEOUT)
        goto done;
    if (!TerminateProcess(owner.hProcess, 73) || WaitForSingleObject(owner.hProcess, 5000) != WAIT_OBJECT_0 ||
        !GetExitCodeProcess(owner.hProcess, &owner_exit) || owner_exit != 73 ||
        WaitForSingleObject(leaf, 5000) != WAIT_OBJECT_0 || !GetExitCodeProcess(leaf, &leaf_exit) ||
        leaf_exit == 47)
        goto done;
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION counts;
    ULONGLONG until = GetTickCount64() + 3000;
    do {
        if (!QueryInformationJobObject(accounting, JobObjectBasicAccountingInformation,
                                       &counts, sizeof(counts), NULL))
            goto done;
        if (!counts.ActiveProcesses) break;
        Sleep(20);
    } while (GetTickCount64() < until);
    ok = counts.TotalProcesses == 1 && counts.ActiveProcesses == 0;
    if (ok) printf("starter-crash ready 1 active-before 1 starter-exit %lu leaf-exit %lu active-after 0 total 1\n",
                   owner_exit, leaf_exit);
done:
    if (owner.hProcess) {
        if (!ok) {
            TerminateProcess(owner.hProcess, 74);
            WaitForSingleObject(owner.hProcess, 5000);
            TerminateJobObject(accounting, 74);
        }
        CloseHandle(owner.hProcess);
        CloseHandle(owner.hThread);
    }
    if (leaf) CloseHandle(leaf);
    if (initialized) DeleteProcThreadAttributeList(attrs);
    if (attrs) HeapFree(GetProcessHeap(), 0, attrs);
    if (signal) CloseHandle(signal);
    if (ready) CloseHandle(ready);
    if (accounting) CloseHandle(accounting);
    return ok;
}

int wmain(int argc, wchar_t **argv) {
    BOOL owner_mode = argc == 5 && wcscmp(argv[1], L"--crash-owner") == 0;
    if (!owner_mode && (argc != 2 || wcscmp(argv[1], L"--prepared-offline-guest") != 0))
        return 2;
    wchar_t directory[MAX_PATH], worker[MAX_PATH], foreign[MAX_PATH], allowed_dll[MAX_PATH],
        foreign_dll[MAX_PATH], input_path[MAX_PATH];
    DWORD length = GetModuleFileNameW(NULL, directory, MAX_PATH);
    wchar_t *base = length && length < MAX_PATH ? wcsrchr(directory, L'\\') : NULL;
    if (!base || wcscmp(base + 1, L"bp3-starter.exe") != 0)
        return 2;
    *base = 0;
    if (wcscmp(directory, L"C:\\BP3-UMCI-Fixture") != 0 ||
        FAILED(StringCchPrintfW(worker, MAX_PATH, L"%s\\bp3-worker.exe", directory)) ||
        FAILED(StringCchPrintfW(foreign, MAX_PATH, L"%s\\foreign.exe", directory)) ||
        FAILED(StringCchPrintfW(allowed_dll, MAX_PATH, L"%s\\allowed.dll", directory)) ||
        FAILED(StringCchPrintfW(foreign_dll, MAX_PATH, L"%s\\foreign.dll", directory)) ||
        FAILED(StringCchPrintfW(input_path, MAX_PATH, L"%s\\input.bin", directory)))
        return 2;
    DWORD attributes = GetFileAttributesW(directory);
    if (attributes == INVALID_FILE_ATTRIBUTES || (attributes & FILE_ATTRIBUTE_REPARSE_POINT))
        return 2;
    if (owner_mode) {
        wchar_t *end;
        HANDLE accounting = (HANDLE)(uintptr_t)_wcstoui64(argv[2], &end, 10);
        if (!accounting || *end || wcsncmp(argv[4], L"org.kraqel.bp3umci.", 19)) return 2;
        HANDLE signal = (HANDLE)(uintptr_t)_wcstoui64(argv[3], &end, 10);
        if (!signal || *end) return 2;
        probe held = { NULL, accounting, NULL, worker, directory };
        if (FAILED(DeriveAppContainerSidFromAppContainerName(argv[4], &held.sid))) return 4;
        held.job = CreateJobObjectW(NULL, NULL);
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION limits = {0};
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        limits.BasicLimitInformation.ActiveProcessLimit = 1;
        if (held.job && SetInformationJobObject(held.job, JobObjectExtendedLimitInformation, &limits, sizeof(limits)))
            role(&held, signal, CRASH_LEAF);
        if (held.job) CloseHandle(held.job);
        FreeSid(held.sid);
        return 4; /* Normal return cannot satisfy the crash assertion. */
    }
    STARTUPINFOW si = {0};
    si.cb = sizeof(si);
    PROCESS_INFORMATION p = {0};
    wchar_t command[] = L"foreign";
    if (CreateProcessW(foreign, command, NULL, NULL, FALSE, CREATE_SUSPENDED,
                       NULL, directory, &si, &p)) {
        TerminateProcess(p.hProcess, 2);
        WaitForSingleObject(p.hProcess, 2000);
        CloseHandle(p.hThread);
        CloseHandle(p.hProcess);
        return 3;
    }
    DWORD rejection = GetLastError();
    printf("foreign-exe win32 %lu\n", rejection);
    if (rejection != ERROR_INVALID_IMAGE_HASH && rejection != ERROR_SYSTEM_INTEGRITY_POLICY_VIOLATION)
        return 3;
    wchar_t profile[64];
    if (FAILED(StringCchPrintfW(profile, 64, L"org.kraqel.bp3umci.%lu.%llu",
                               GetCurrentProcessId(), GetTickCount64())))
        return 4;
    probe test = { NULL, NULL, NULL, worker, directory };
    if (FAILED(CreateAppContainerProfile(profile, profile, profile, NULL, 0, &test.sid)))
        return 4;
    HANDLE input = INVALID_HANDLE_VALUE;
    BOOL ok = FALSE;
    if (allow_read(directory, test.sid) || allow_read(worker, test.sid) ||
        allow_read(allowed_dll, test.sid) || allow_read(foreign_dll, test.sid))
        goto finish;
    SECURITY_ATTRIBUTES inherit = {sizeof(inherit), NULL, TRUE};
    input = CreateFileW(input_path, GENERIC_READ, FILE_SHARE_READ, &inherit, OPEN_EXISTING,
                        FILE_FLAG_OPEN_REPARSE_POINT, NULL);
    if (input == INVALID_HANDLE_VALUE)
        goto finish;
    test.job = CreateJobObjectW(NULL, NULL);
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limit = {0};
    limit.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    limit.BasicLimitInformation.ActiveProcessLimit = 4;
    if (!test.job || !SetInformationJobObject(test.job, JobObjectExtendedLimitInformation,
                                             &limit, sizeof(limit)))
        goto finish;
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
    ok = role(&test, input, LEAF) && role(&test, NULL, LEAF);
    wchar_t starter[MAX_PATH];
    if (ok) ok = GetModuleFileNameW(NULL, starter, MAX_PATH) == length && crash_trial(starter, profile, &test);
    const wchar_t *images[] = { L"bp3-static-allowed.exe", L"bp3-static-foreign.exe" };
    for (unsigned i = 0; ok && i < 2; ++i) {
        ok = SUCCEEDED(StringCchPrintfW(worker, MAX_PATH, L"%s\\%s", directory, images[i])) &&
             allow_read(worker, test.sid) == ERROR_SUCCESS &&
             role(&test, NULL, i == 0 ? STATIC_ALLOWED : STATIC_FOREIGN);
    }
finish:
    if (input != INVALID_HANDLE_VALUE) CloseHandle(input);
    BOOL drained = TRUE;
    if (test.job) {
        if (!ok) TerminateJobObject(test.job, 2);
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION accounting;
        ULONGLONG deadline = GetTickCount64() + 3000;
        do {
            drained = QueryInformationJobObject(test.job, JobObjectBasicAccountingInformation,
                                                &accounting, sizeof(accounting), NULL) &&
                accounting.ActiveProcesses == 0;
            if (!drained) Sleep(25);
        } while (!drained && GetTickCount64() < deadline);
        CloseHandle(test.job);
    }
    HRESULT removed = drained ? DeleteAppContainerProfile(profile) : E_FAIL;
    FreeSid(test.sid);
    printf("job-drained %u profile-removed %u\n", drained != FALSE, SUCCEEDED(removed));
    return ok && drained && SUCCEEDED(removed) ? 0 : 4;
}
