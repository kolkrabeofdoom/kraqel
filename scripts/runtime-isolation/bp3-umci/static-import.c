#define WIN32_LEAN_AND_MEAN
#include <windows.h>

__declspec(dllimport) DWORD WINAPI Bp3Marker(void);

int wmain(void) {
    return Bp3Marker() == 0x42503331 ? 50 : 51;
}
