#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdint.h>
#include <strsafe.h>
#include <stdlib.h>

/* Roles use the exact same image. Only one receives the synthetic input handle. */
enum { OK = 40, BAD_CONTEXT = 41, BAD_RIGHTS = 42, BAD_CHILD = 43,
       BAD_DLL = 44, BAD_CODE = 45, FOREIGN_EXE_RAN = 46 };

#ifdef BP3_FOREIGN
int wmain(void) { return FOREIGN_EXE_RAN; }
#else
typedef DWORD (WINAPI *marker_proc)(void);

static BOOL rejected_dll(const wchar_t *path) {
    HMODULE module = LoadLibraryExW(path, NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
    if (!module) {
        DWORD error = GetLastError();
        return error == ERROR_INVALID_IMAGE_HASH || error == ERROR_SYSTEM_INTEGRITY_POLICY_VIOLATION;
    }
    FreeLibrary(module);
    return FALSE;
}

static int child_blocked(const wchar_t *image, DWORD flag) {
    STARTUPINFOW si = {0};
    PROCESS_INFORMATION p = {0};
    wchar_t command[] = L"bp3-worker";
    si.cb = sizeof(si);
    if (!CreateProcessW(image, command, NULL, NULL, FALSE,
                        CREATE_SUSPENDED | CREATE_NO_WINDOW | flag,
                        NULL, NULL, &si, &p))
        return GetLastError() == ERROR_CHILD_PROCESS_BLOCKED;
    TerminateProcess(p.hProcess, BAD_CHILD);
    DebugActiveProcessStop(p.dwProcessId);
    CloseHandle(p.hThread);
    CloseHandle(p.hProcess);
    return 0;
}

int wmain(int argc, wchar_t **argv) {
    HANDLE token;
    DWORD container = 0, bytes = 0, count = 0;
    BOOL member = FALSE;
    wchar_t image[MAX_PATH], path[MAX_PATH], *leaf;
    PROCESS_MITIGATION_DYNAMIC_CODE_POLICY dynamic;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &token))
        return BAD_CONTEXT;
    BOOL queried = GetTokenInformation(token, TokenIsAppContainer, &container,
                                        sizeof(container), &bytes);
    CloseHandle(token);
    if (!queried || !container ||
        !IsProcessInJob(GetCurrentProcess(), NULL, &member) || !member ||
        argc != 2 || !GetModuleFileNameW(NULL, image, MAX_PATH))
        return BAD_CONTEXT;
    HANDLE by_path = CreateFileW(L"C:\\BP3-UMCI-Fixture\\input.bin", GENERIC_READ,
                                 FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    if (by_path != INVALID_HANDLE_VALUE) {
        CloseHandle(by_path);
        return BAD_RIGHTS;
    }
    if (GetLastError() != ERROR_ACCESS_DENIED)
        return BAD_RIGHTS;
    if (wcscmp(argv[1], L"none") != 0) {
        wchar_t *end = NULL;
        uintptr_t value = (uintptr_t)_wcstoui64(argv[1], &end, 10);
        char marker = 0;
        if (!value || end == argv[1] || *end ||
            !SetHandleInformation((HANDLE)value, HANDLE_FLAG_INHERIT, 0) ||
            !ReadFile((HANDLE)value, &marker, 1, &count, NULL) ||
            count != 1 || marker != 'K')
            return BAD_RIGHTS;
        CloseHandle((HANDLE)value);
    }
    if (!child_blocked(image, 0) || !child_blocked(image, DEBUG_PROCESS))
        return BAD_CHILD;
    if (!GetProcessMitigationPolicy(GetCurrentProcess(), ProcessDynamicCodePolicy,
                                    &dynamic, sizeof(dynamic)) ||
        !dynamic.ProhibitDynamicCode || dynamic.AllowThreadOptOut ||
        dynamic.AllowRemoteDowngrade || dynamic.AuditProhibitDynamicCode)
        return BAD_CODE;
    void *page = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (page) {
        VirtualFree(page, 0, MEM_RELEASE);
        return BAD_CODE;
    }
    if (GetLastError() != ERROR_DYNAMIC_CODE_BLOCKED)
        return BAD_CODE;
    if (FAILED(StringCchCopyW(path, MAX_PATH, image)) || !(leaf = wcsrchr(path, L'\\')))
        return BAD_CONTEXT;
    ++leaf;
    size_t space = MAX_PATH - (size_t)(leaf - path);
    if (FAILED(StringCchCopyW(leaf, space, L"foreign.dll")) || !rejected_dll(path))
        return BAD_DLL;
    if (FAILED(StringCchCopyW(leaf, space, L"allowed.dll")))
        return BAD_CONTEXT;
    HMODULE module = LoadLibraryExW(path, NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
    if (!module)
        return BAD_DLL;
    marker_proc mark = (marker_proc)GetProcAddress(module, "Bp3Marker");
    if (!mark || mark() != 0x42503331)
        return BAD_DLL;
    FreeLibrary(module);
    if (FAILED(StringCchCopyW(leaf, space, L"foreign.dll")) || !rejected_dll(path))
        return BAD_DLL;
    return OK;
}
#endif
