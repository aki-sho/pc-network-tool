using System;
using System.Runtime.InteropServices;

namespace PcNetworkTool
{
    // Keep Windows' Authenticode integrity checks, but expose the exact error so
    // local self-signed builds can distinguish an untrusted root from corruption.
    public static class AuthenticodeVerification
    {
        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        private struct FileInfo
        {
            public uint Size;
            [MarshalAs(UnmanagedType.LPWStr)] public string Path;
            public IntPtr File;
            public IntPtr KnownSubject;
        }

        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        private struct TrustData
        {
            public uint Size;
            public IntPtr PolicyCallback;
            public IntPtr SipClient;
            public uint UiChoice;
            public uint RevocationChecks;
            public uint UnionChoice;
            public IntPtr FileInfo;
            public uint StateAction;
            public IntPtr StateData;
            public IntPtr UrlReference;
            public uint ProviderFlags;
            public uint UiContext;
            public IntPtr SignatureSettings;
        }

        [DllImport("wintrust.dll", ExactSpelling = true, CharSet = CharSet.Unicode)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        private static extern int WinVerifyTrust(IntPtr window, ref Guid action, ref TrustData data);

        public static int Verify(string path)
        {
            var file = new FileInfo { Size = (uint)Marshal.SizeOf(typeof(FileInfo)), Path = path };
            IntPtr filePointer = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(FileInfo)));
            Marshal.StructureToPtr(file, filePointer, false);
            var data = new TrustData
            {
                Size = (uint)Marshal.SizeOf(typeof(TrustData)),
                UiChoice = 2, // WTD_UI_NONE
                UnionChoice = 1, // WTD_CHOICE_FILE
                FileInfo = filePointer,
                StateAction = 1, // WTD_STATEACTION_VERIFY
                ProviderFlags = 0x1000 // WTD_CACHE_ONLY_URL_RETRIEVAL
            };
            var action = new Guid("00AAC56B-CD44-11d0-8CC2-00C04FC295EE");
            try
            {
                return WinVerifyTrust(new IntPtr(-1), ref action, ref data);
            }
            finally
            {
                data.StateAction = 2; // WTD_STATEACTION_CLOSE
                WinVerifyTrust(new IntPtr(-1), ref action, ref data);
                Marshal.DestroyStructure(filePointer, typeof(FileInfo));
                Marshal.FreeHGlobal(filePointer);
            }
        }
    }
}
