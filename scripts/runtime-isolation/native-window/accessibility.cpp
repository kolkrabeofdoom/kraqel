#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <ole2.h>
#include <oleacc.h>
#include <uiautomation.h>
#include <wrl/client.h>
#include <cwchar>
using Microsoft::WRL::ComPtr;

// Reads only the two controls of the pinned synthetic guest image.
static bool matches(IUIAutomation* automation, HWND target, CONTROLTYPEID wanted, const wchar_t* label) {
    ComPtr<IUIAutomationElement> element;
    if (!target || FAILED(automation->ElementFromHandle(target, &element))) return false;
    CONTROLTYPEID kind = 0;
    BOOL focused = FALSE;
    BSTR name = nullptr;
    if (FAILED(element->get_CurrentControlType(&kind)) ||
        FAILED(element->get_CurrentHasKeyboardFocus(&focused)) ||
        FAILED(element->get_CurrentName(&name))) return false;
    bool ok = focused && kind == wanted && name && wcscmp(name, label) == 0;
    SysFreeString(name);
    return ok;
}
static int observe() {
    ComPtr<IUIAutomation> automation;
    if (FAILED(CoCreateInstance(CLSID_CUIAutomation, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(&automation)))) return 5;
    bool edit = false;
    const ULONGLONG end = GetTickCount64() + 240000;
    while (GetTickCount64() < end) {
        HWND window = FindWindowW(L"KRAQEL.Synthetic.Edit", L"KRAQEL – Offline-Fensterprobe");
        if (!window && edit) break;
        DWORD pid = 0;
        if (window) GetWindowThreadProcessId(window, &pid);
        HANDLE process = pid ? OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) : nullptr;
        if (process) {
            wchar_t image[MAX_PATH];
            DWORD n = MAX_PATH;
            const bool bound = QueryFullProcessImageNameW(process, 0, image, &n) &&
                _wcsicmp(image,L"C:\\Program Files\\KRAQEL-Window-Probe\\native-leaf.exe") == 0;
            if (bound && !edit) edit = matches(automation.Get(), GetDlgItem(window,101), UIA_EditControlTypeId, L"Prüftext");
            bool button = bound && edit && matches(automation.Get(),GetDlgItem(window,102), UIA_ButtonControlTypeId,L"Schließen");
            CloseHandle(process);
            if (button) return 0;
        }
        Sleep(150);
    }
    return edit ? 7 : 6;
}
int WINAPI wWinMain(HINSTANCE, HINSTANCE, PWSTR, int) {
    if (FAILED(CoInitializeEx(nullptr,COINIT_MULTITHREADED))) return 4;
    const int status = observe();
    CoUninitialize();
    return status;
}
