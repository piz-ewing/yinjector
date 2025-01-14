#include "rewolf-wow64ext/src/wow64ext.h"
#include <string>

typedef struct _UNICODE_STRING
{
    USHORT Length; // UNICODE占用的内存字节数
    USHORT MaximumLength;
    DWORD64 Buffer; // 注意这里指针的问题
} UNICODE_STRING, *PUNICODE_STRING;

unsigned char shell_code_ex[] = {
    0x48, 0x89, 0x4c, 0x24, 0x08,                               // mov       qword ptr [rsp+8],rcx
    0x57,                                                       // push      rdi
    0x48, 0x83, 0xec, 0x20,                                     // sub       rsp,20h
    0x48, 0x8b, 0xfc,                                           // mov       rdi,rsp
    0xb9, 0x08, 0x00, 0x00, 0x00,                               // mov       ecx,8
    0xb8, 0xcc, 0xcc, 0xcc, 0xcc,                               // mov       eac,0CCCCCCCCh
    0xf3, 0xab,                                                 // rep stos  dword ptr [rdi]
    0x48, 0x8b, 0x4c, 0x24, 0x30,                               // mov       rcx,qword ptr [__formal]
    0x49, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       r9,0  //PVOID*  BaseAddr opt
    0x49, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       r8,0  //PUNICODE_STRING Name
    0x48, 0xba, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       rdx,0
    0x48, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       rcx,0
    0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       rax,0
    0xff, 0xd0,                                                 // call      rax   LdrLoadDll
    0x48, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       rcx,0
    0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov       rax,0
    0xff, 0xd0                                                  // call      rax
};

const static int cs_paramemter_size = (MAX_PATH + 1) * sizeof(TCHAR) + sizeof(UNICODE_STRING) + sizeof(DWORD64);

std::wstring local_codepage_to_utf16(std::string input)
{
    size_t need = MultiByteToWideChar(CP_ACP, 0, input.c_str(), input.size(), 0, 0);
    wchar_t *wstr = static_cast<wchar_t *>(malloc(need + sizeof(wchar_t)));
    size_t used = MultiByteToWideChar(CP_ACP, 0, input.c_str(), input.size(), wstr, need);
    std::wstring result(wstr, used);
    free(wstr);
    return result;
}

extern "C" __declspec(dllexport) int __cdecl inject64(HANDLE hProcess, const char *cdll, int timeout /* = INFINITE*/)
{
    BOOL bOk = FALSE;
    BOOL bClean = TRUE;

    std::wstring dll = local_codepage_to_utf16(cdll);

    try
    {
        DWORD64 hRemoteThread = 0;
        DWORD64 paramemter_mem_addr = 0;
        DWORD64 shell_code_addr = 0;

        do
        {
            paramemter_mem_addr = (DWORD64)VirtualAllocEx64(hProcess, NULL, cs_paramemter_size, MEM_COMMIT, PAGE_READWRITE);
            shell_code_addr = (DWORD64)VirtualAllocEx64(hProcess, NULL, sizeof(shell_code_ex), MEM_COMMIT, PAGE_EXECUTE_READWRITE);

            if (!paramemter_mem_addr || !shell_code_addr)
            {
                break;
            }

            char paramemter_mem_local[cs_paramemter_size] = {0};
            PUNICODE_STRING ptr_unicode_string = (PUNICODE_STRING)(paramemter_mem_local + sizeof(DWORD64));
            size_t file_path_mem_length = dll.length();
            ptr_unicode_string->Length = file_path_mem_length * 2;
            ptr_unicode_string->MaximumLength = file_path_mem_length * 2;
            wcscpy_s((WCHAR *)(ptr_unicode_string + 1), MAX_PATH, dll.c_str());
            ptr_unicode_string->Buffer = (DWORD64)(paramemter_mem_addr + sizeof(DWORD64) + sizeof(UNICODE_STRING));
            WCHAR szDll[] = {L"ntdll.dll"};
            char szFunc[] = {"LdrLoadDll"};
            char szThread[] = {"RtlCreateUserThread"};
            char szExitThread[] = {"RtlExitUserThread"};
            DWORD64 ntdll64 = GetModuleHandle64(szDll);
            DWORD64 ntdll_LdrLoadDll = GetProcAddress64(ntdll64, szFunc);
            DWORD64 ntdll_RtlCreateUserThread = GetProcAddress64(ntdll64, szThread);
            DWORD64 ntdll_RtlExitThread = GetProcAddress64(ntdll64, szExitThread);

            if (NULL == ntdll_LdrLoadDll || NULL == ntdll_RtlCreateUserThread || NULL == ntdll_RtlExitThread)
            {
                break;
            }

            // r9
            memcpy(shell_code_ex + 32, &paramemter_mem_addr, sizeof(DWORD64));
            // r8
            DWORD64 ptr = paramemter_mem_addr + sizeof(DWORD64);
            memcpy(shell_code_ex + 42, &ptr, sizeof(DWORD64));
            // LdrLoaddll
            memcpy(shell_code_ex + 72, &ntdll_LdrLoadDll, sizeof(DWORD64));
            // RtlExitUserThread
            memcpy(shell_code_ex + 94, &ntdll_RtlExitThread, sizeof(DWORD64));
            size_t write_size = 0;

            if (!WriteProcessMemory64(hProcess, paramemter_mem_addr, paramemter_mem_local, cs_paramemter_size, NULL) ||
                !WriteProcessMemory64(hProcess, shell_code_addr, shell_code_ex, sizeof(shell_code_ex), NULL))
            {
                break;
            }

            struct
            {
                DWORD64 UniqueProcess;
                DWORD64 UniqueThread;
            } client_id = {0};
            DWORD64 ret64 = X64Call(ntdll_RtlCreateUserThread, 10,
                                    (DWORD64)hProcess,       // ProcessHandle
                                    (DWORD64)NULL,           // SecurityDescriptor
                                    (DWORD64)FALSE,          // CreateSuspended
                                    (DWORD64)0,              // StackZeroBits
                                    (DWORD64)NULL,           // StackReserved
                                    (DWORD64)NULL,           // StackCommit
                                    shell_code_addr,         // StartAddress
                                    (DWORD64)NULL,           // StartParameter
                                    (DWORD64)&hRemoteThread, // ThreadHandle
                                    (DWORD64)&client_id);    // ClientID)

            if (INVALID_HANDLE_VALUE == (HANDLE)hRemoteThread)
            {
                break;
            }

            // 等待远程线程执行结束
            DWORD dwRet = WaitForSingleObject((HANDLE)hRemoteThread, timeout);

            if (WAIT_FAILED == dwRet)
            {
                break;
            }
            else if (WAIT_TIMEOUT == dwRet)
            {
                bClean = FALSE;
                break;
            }

            bOk = TRUE;
        } while (0);

        if (bClean && paramemter_mem_addr)
            VirtualFreeEx64(hProcess, paramemter_mem_addr, cs_paramemter_size, MEM_DECOMMIT);

        if (bClean && shell_code_addr)
            VirtualFreeEx64(hProcess, shell_code_addr, sizeof(shell_code_ex), MEM_DECOMMIT);
    }
    catch (...)
    {
    }

    return bOk;
}