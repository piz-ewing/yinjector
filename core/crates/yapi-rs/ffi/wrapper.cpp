#include "yapi/yapi.hpp"
#include <minwindef.h>
#include <wow64apiset.h>

using namespace yapi;

extern "C"
{
  DWORD64 yinject(HANDLE hProcess, const char *dll_path, int is_wow64)
  {
    YAPICall LoadLibraryA(hProcess, _T("kernel32.dll"), "LoadLibraryA");
    return is_wow64 ? LoadLibraryA(dll_path) : LoadLibraryA.Dw64()(dll_path);
  }
}