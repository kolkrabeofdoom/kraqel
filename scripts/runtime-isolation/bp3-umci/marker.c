#define WIN32_LEAN_AND_MEAN
#include <windows.h>

__declspec(dllexport) DWORD WINAPI Bp3Marker(void) {
#ifdef BP3_FOREIGN
    return 0x42503332;
#else
    return 0x42503331;
#endif
}
