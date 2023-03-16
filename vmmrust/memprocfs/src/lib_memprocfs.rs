//! # The MemProcFS API Documentation
//!
//! The MemProcFS crate contains a wrapper API around the [MemProcFS physical
//! memory analysis framework](https://github.com/ufrisk/MemProcFS). The native
//! libray in the form of `vmm.dll` or `vmm.so` must be downloaded or compiled
//! in order to make use of the memprocfs crate.
//! 
//! Physical memory analysis may take place on memory dump files for forensic
//! purposes. Analysis may also take place on live memory - either captured by
//! using [PCILeech PCIe DMA devices](https://github.com/ufrisk/pcileech-fpga)
//! or by using a driver - such as WinPMEM, LiveCloudKd, VMware or similar.
//! 
//! The base of the MemProcFS API is the [`Vmm`] struct. Once the native vmm
//! has been initialized it's possible to retrieve processes in the form of
//! the [`VmmProcess`] struct. Using the `Vmm` and `VmmProcess` it's possible
//! to undertake a wide range of actions - such as reading/writing memory or
//! retrieve various information.
//! 
//! 
//! <b>Read and write memory</b> by using the methods
//! [`mem_read()`](VmmProcess::mem_read()),
//! [`mem_read_ex()`](VmmProcess::mem_read_ex()) and
//! [`mem_write()`](VmmProcess::mem_write()).
//! Virtual memory is read from [`VmmProcess`] struct.
//! Physical memory is read from the [`Vmm`] struct.
//! 
//! <b>Efficiently read and write memory</b> using the [`VmmScatterMemory`]
//! struct. The scatter struct is retrieved by calling
//! [`mem_scatter()`](VmmProcess::mem_scatter()) on either the base [`Vmm`]
//! struct or the individual [`VmmProcess`] structs.
//! 
//! <b>Access information</b> about loaded modules, memory regions, registry,
//! process handles, kernel pool allocations and much more!
//! 
//! <b>Access the Virtual File System</b> (VFS) using the Rust API to get access
//! to the full range of built-in and external plugins. The VFS is accessed by
//! using the methods
//! [`vfs_list()`](Vmm::vfs_list()), [`vfs_read()`](Vmm::vfs_read()) and
//! [`vfs_write()`](Vmm::vfs_write()) on the [`Vmm`] struct.
//! 
//! The MemProcFS crate and API also supports creation of native MemProcFS
//! plugins in the form of a library `.dll` or `.so`.
//! 
//! 
//! ## Example projects
//! Check out the
//! [Example Project](https://github.com/ufrisk/MemProcFS/blob/master/vmmrust/memprocfs_example/src/main.rs)
//! and the
//! [Example Plugin](https://github.com/ufrisk/MemProcFS/blob/master/vmmrust/m_example_plugin/src/lib.rs).
//! 
//! 
//! ## Project documentation
//! Check out the project documentation for MemProcFS, LeechCore and pcileech-fpga:
//! * [MemProcFS](https://github.com/ufrisk/MemProcFS) - [Documentation](https://github.com/ufrisk/MemProcFS/wiki).
//! * [LeechCore](https://github.com/ufrisk/LeechCore/) - [Documentation](https://github.com/ufrisk/LeechCore/wiki).
//! * [PCILeech](https://github.com/ufrisk/pcileech) - [Documentation](https://github.com/ufrisk/pcileech/wiki).
//! * [PCILeech-FPGA](https://github.com/ufrisk/pcileech-fpga).
//! 
//! 
//! ## Support PCILeech/MemProcFS development:
//! PCILeech and MemProcFS is free and open source!
//! 
//! I put a lot of time and energy into PCILeech and MemProcFS and related
//! research to make this happen. Some aspects of the projects relate to
//! hardware and I put quite some money into my projects and related research.
//! If you think PCILeech and/or MemProcFS are awesome tools and/or if you
//! had a use for them it's now possible to contribute by becoming a sponsor!
//! 
//! If you like what I've created with PCIleech and MemProcFS with regards to
//! DMA, Memory Analysis and Memory Forensics and would like to give something
//! back to support future development please consider becoming a sponsor at:
//! <https://github.com/sponsors/ufrisk>
//! 
//! To all my sponsors, Thank You 💖
//! 
//! 
//! ## Questions and Comments
//! Please feel free to contact me!
//! * Github: <https://github.com/ufrisk/MemProcFS>
//! * Discord #pcileech channel at the [Porchetta](https://discord.gg/sEkn3aa) server.
//! * Twitter: <https://twitter.com/UlfFrisk>
//! * Email: pcileech@frizk.net
//! 
//! 
//! ## Get Started!
//! Check out the [`Vmm`] documentation and the
//! [Example Project](https://github.com/ufrisk/MemProcFS/tree/master/vmmrust/memprocfs_example)!
//! 
//! <b>Best wishes with your memory analysis project!</b>

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int};
use std::fmt;
use serde::{Serialize, Deserialize};



/// Result type for MemProcFS API.
/// 
/// The MemProcFS result type contains a function-defined return type and
/// a String error type.
pub type ResultEx<T> = std::result::Result<T, Box<dyn std::error::Error>>;



// MemProcFS memory read/write flags:
/// Do not use internal data cache.
pub const FLAG_NOCACHE                              : u64 = 0x0001;
/// Zero pad failed memory reads and report success.
pub const FLAG_ZEROPAD_ON_FAIL                      : u64 = 0x0002;
/// Force use of data cache - fail non-cached pages.
///
/// Flag is only valid for reads, invalid with VMM_FLAG_NOCACHE/VMM_FLAG_ZEROPAD_ON_FAIL.
pub const FLAG_FORCECACHE_READ                      : u64 = 0x0008;
/// Do not retrieve memory from paged out memory.
/// 
/// Paged out memory may be from pagefile/compressed (even if possible).
/// If slow I/O accesses are the concern the flag `FLAG_NOPAGING_IO` may be a better choice.
pub const FLAG_NOPAGING                             : u64 = 0x0010;
/// Do not retrieve memory from paged out memory***.
/// 
/// ***) If the read would incur additional I/O (even if possible).
pub const FLAG_NOPAGING_IO                          : u64 = 0x0020;
/// Do not populate the data cache on a successful read.
pub const FLAG_NOCACHEPUT                           : u64 = 0x0100;
/// Only fetch from the most recent active cache region when reading.
pub const FLAG_CACHE_RECENT_ONLY                    : u64 = 0x0200;
/// Do not perform additional predictive page reads.
///
/// This is default on smaller requests.
pub const FLAG_NO_PREDICTIVE_READ                   : u64 = 0x0400;
/// Disable/override any use of VMM_FLAG_FORCECACHE_READ.
/// 
/// This flag is only recommended for local files. improves forensic artifact order.
pub const FLAG_FORCECACHE_READ_DISABLE              : u64 = 0x0800;
/// Get/Set library console printouts.
pub const CONFIG_OPT_CORE_PRINTF_ENABLE             : u64 = 0x4000000100000000;
/// Get/Set standard verbosity.
pub const CONFIG_OPT_CORE_VERBOSE                   : u64 = 0x4000000200000000;
/// Get/Set extra verbosity.
pub const CONFIG_OPT_CORE_VERBOSE_EXTRA             : u64 = 0x4000000300000000;
/// Get/Set super extra verbosity and PCIe TLP debug.
pub const CONFIG_OPT_CORE_VERBOSE_EXTRA_TLP         : u64 = 0x4000000400000000;
/// Get max native physical memory address.
pub const CONFIG_OPT_CORE_MAX_NATIVE_ADDRESS        : u64 = 0x4000000800000000;
/// Get the numeric system type according to VMM C-API.
pub const CONFIG_OPT_CORE_SYSTEM                    : u64 = 0x2000000100000000;
/// Get the numeric memory model type according to the VMM C-API.
pub const CONFIG_OPT_CORE_MEMORYMODEL               : u64 = 0x2000000200000000;
/// Get whether the refresh is enabled or not (1/0).
pub const CONFIG_OPT_CONFIG_IS_REFRESH_ENABLED      : u64 = 0x2000000300000000;
/// Get/Set base tick period in ms.
pub const CONFIG_OPT_CONFIG_TICK_PERIOD             : u64 = 0x2000000400000000;
/// Get/Set memory cache validity period (in ticks).
pub const CONFIG_OPT_CONFIG_READCACHE_TICKS         : u64 = 0x2000000500000000;
/// Get/Set page table (tlb) cache validity period (in ticks).
pub const CONFIG_OPT_CONFIG_TLBCACHE_TICKS          : u64 = 0x2000000600000000;
/// Get/Set process refresh (partial) period (in ticks).
pub const CONFIG_OPT_CONFIG_PROCCACHE_TICKS_PARTIAL : u64 = 0x2000000700000000;
/// Get/Set process refresh (full) period (in ticks).
pub const CONFIG_OPT_CONFIG_PROCCACHE_TICKS_TOTAL   : u64 = 0x2000000800000000;
/// Get MemProcFS major version.
pub const CONFIG_OPT_CONFIG_VMM_VERSION_MAJOR       : u64 = 0x2000000900000000;
/// Get MemProcFS minor version.
pub const CONFIG_OPT_CONFIG_VMM_VERSION_MINOR       : u64 = 0x2000000A00000000;
/// Get MemProcFS revision version.
pub const CONFIG_OPT_CONFIG_VMM_VERSION_REVISION    : u64 = 0x2000000B00000000;
/// Get/Set enable function call statistics (.status/statistics_fncall file).
pub const CONFIG_OPT_CONFIG_STATISTICS_FUNCTIONCALL : u64 = 0x2000000C00000000;
/// Get/Set enable paging support 1/0.
pub const CONFIG_OPT_CONFIG_IS_PAGING_ENABLED       : u64 = 0x2000000D00000000;
/// Set native library internal custom debug.
pub const CONFIG_OPT_CONFIG_DEBUG                   : u64 = 0x2000000E00000000;
/// Get OS version major.
pub const CONFIG_OPT_WIN_VERSION_MAJOR              : u64 = 0x2000010100000000;
/// Get OS version minor.
pub const CONFIG_OPT_WIN_VERSION_MINOR              : u64 = 0x2000010200000000;
/// Get OS version build.
pub const CONFIG_OPT_WIN_VERSION_BUILD              : u64 = 0x2000010300000000;
/// Get MemProcFS unique system id.
pub const CONFIG_OPT_WIN_SYSTEM_UNIQUE_ID           : u64 = 0x2000010400000000;
/// Get/Set enable/retrieve forensic mode type [0-4].
pub const CONFIG_OPT_FORENSIC_MODE                  : u64 = 0x2000020100000000;

// REFRESH OPTIONS:
/// Set - trigger refresh all caches.
pub const CONFIG_OPT_REFRESH_ALL                    : u64 = 0x2001ffff00000000;
/// Set - refresh memory cache (excl. TLB) (fully).
pub const CONFIG_OPT_REFRESH_FREQ_MEM               : u64 = 0x2001100000000000;
/// Set - refresh memory cache (excl. TLB) [partial 33%/call].
pub const CONFIG_OPT_REFRESH_FREQ_MEM_PARTIAL       : u64 = 0x2001000200000000;
/// Set - refresh page table (TLB) cache (fully)
pub const CONFIG_OPT_REFRESH_FREQ_TLB               : u64 = 0x2001080000000000;
/// Set - refresh page table (TLB) cache [partial 33%/call].
pub const CONFIG_OPT_REFRESH_FREQ_TLB_PARTIAL       : u64 = 0x2001000400000000;
/// Set - refresh fast frequency - incl. partial process refresh.
pub const CONFIG_OPT_REFRESH_FREQ_FAST              : u64 = 0x2001040000000000;
/// Set - refresh medium frequency - incl. full process refresh.
pub const CONFIG_OPT_REFRESH_FREQ_MEDIUM            : u64 = 0x2001000100000000;
/// Set - refresh slow frequency.
pub const CONFIG_OPT_REFRESH_FREQ_SLOW              : u64 = 0x2001001000000000;
/// Set custom process directory table base. [LO-DWORD: Process PID].
pub const CONFIG_OPT_PROCESS_DTB                    : u64 = 0x2002000100000000;

// PLUGIN NOTIFICATIONS:
/// Verbosity change. Query new verbosity with: `vmm.get_config()`.
pub const PLUGIN_NOTIFY_VERBOSITYCHANGE             : u32 = 0x01;
/// Fast refresh. Partial process refresh.
pub const PLUGIN_NOTIFY_REFRESH_FAST                : u32 = 0x05;
/// Medium refresh. Full process refresh and other refresh tasks.
pub const PLUGIN_NOTIFY_REFRESH_MEDIUM              : u32 = 0x02;
/// Slow refresh. Total refresh of as much as possible.
pub const PLUGIN_NOTIFY_REFRESH_SLOW                : u32 = 0x04;
/// Forensic mode initialization start.
pub const PLUGIN_NOTIFY_FORENSIC_INIT               : u32 = 0x01000100;
/// Forensic mode processing is completed.
pub const PLUGIN_NOTIFY_FORENSIC_INIT_COMPLETE      : u32 = 0x01000200;
/// A child VM was attached or detached. Query new state with API.
pub const PLUGIN_NOTIFY_VM_ATTACH_DETACH            : u32 = 0x01000400;



/// <b>MemProcFS API Base Struct.</b>
/// 
/// The [`Vmm`] struct is the base of the MemProcFS API. All API accesses
/// takes place from the [`Vmm`] struct and its sub-structs.
/// 
/// The [`Vmm`] struct acts as a wrapper around the native MemProcFS VMM API.
/// 
/// <b>Check out the example project for more detailed API usage and
/// additional examples!</b>
/// 
/// 
/// # Created By
/// - [`Vmm::new()`]
/// - [`Vmm::new_from_virtual_machine()`]
/// - `plugin sub-system`
/// 
/// The [`Vmm`] is normally created by [`Vmm::new()`] (see example below).
/// 
/// The [`Vmm`] object represents memory analysis of a target system. If the
/// target system contains virtual machines additional child `Vmm` objects
/// representing the individual VMs may be retrieved by calling the
/// function [`Vmm::new_from_virtual_machine()`].
/// 
/// The [`Vmm`] object is also supplied by the plugin API to any plugins created.
/// 
/// 
/// # Examples
/// 
/// ```
/// // Initialize MemProcFS VMM on a Windows system parsing a
/// // memory dump and virtual machines inside it.
/// let args = ["-printf", "-v", "-waitinitialize", "-device", "C:\\Dumps\\mem.dmp"].to_vec();
/// if let Ok(vmm) = Vmm::new("C:\\MemProcFS\\vmm.dll", &args) {
///     ...
///     // The underlying native vmm is automatically closed 
///     // when the vmm object goes out of scope.
/// };
/// ```
/// 
/// ```
/// // Initialize MemProcFS VMM on a Linux system parsing live memory
/// // retrieved from a PCILeech FPGA hardware device.
/// let args = ["-device", "fpga"].to_vec();
/// if let Ok(vmm) = Vmm::new("/home/user/memprocfs/vmm.so", &args) {
///     ...
///     // The underlying native vmm is automatically closed 
///     // when the vmm object goes out of scope.
/// };
/// ```
#[allow(dead_code)]
#[derive(Debug)]
pub struct Vmm<'a> {
    native : VmmNative,
    parent_vmm : Option<&'a Vmm<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmLogLevel {
    _1Critical,
    _2Warning,
    _3Info,
    _4Verbose,
    _5Debug,
    _6Trace,
    _7None,
}

/// Info: Network connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapNetEntry {
    pub pid : u32,
    pub state : u32,
    pub address_family : u16,
    pub src_is_valid : bool,
    pub src_port : u16,
    pub src_addr_raw : [u8; 16],
    pub src_str : String,
    pub dst_is_valid : bool,
    pub dst_port : u16,
    pub dst_addr_raw : [u8; 16],
    pub dst_str : String,
    pub va_object : u64,
    pub filetime : u64,
    pub pool_tag : u32,
    pub desc : String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmMapPfnType {
    Zero,
    Free,
    Standby,
    Modified,
    ModifiedNoWrite,
    Bad,
    Active,
    Transition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmMapPfnTypeExtended {
    Unknown,
    Unused,
    ProcessPrivate,
    PageTable,
    LargePage,
    DriverLocked,
    Shareable,
    File,
}

/// Info: Memory PFN (Page Frame Number).
/// 
/// # Created By
/// - `vmmprocess.map_pfn()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapPfnEntry {
    pub pfn : u32,
    pub location : VmmMapPfnType,
    pub is_prototype : bool,
    pub color : u32,
    // extended attributes below - only valid if is_extended == true
    pub is_extended : bool,
    pub tp_ex : VmmMapPfnTypeExtended,
    pub pid : u32,
    pub ptes : [u32; 5],    // 1 = pfn:PTE, .. 4 = pfn:PML4E
    pub va : u64,
    pub va_pte : u64,
    pub pte_original : u64,
}

/// Info: Kernel pool entries.
/// 
/// # Created By
/// - `vmm.map_pool()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapPoolEntry {
    pub va : u64,
    pub cb : u32,
    pub tag : u32,
    pub is_alloc : bool,
    pub tp_pool : u8,           // VMMDLL_MAP_POOL_TYPE
    pub tp_subsegment : u8,     // VMMDLL_MAP_POOL_TYPE_SUBSEGMENT
}

/// Info: Physical memory map entries.
/// 
/// # Created By
/// - `vmm.map_memory()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapMemoryEntry {
    pub pa : u64,
    pub cb : u64
}

/// Info: Services.
/// 
/// # Created By
/// - `vmm.map_service()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapServiceEntry {
    pub ordinal : u32,
    pub va_object : u64,
    pub pid : u32,
    pub start_type : u32,
    pub service_type : u32,
    pub current_state : u32,
    pub controls_accepted : u32,
    pub win32_exit_code : u32,
    pub service_specific_exit_code : u32,
    pub check_point : u32,
    pub wait_hint : u32,
    pub name : String,
    pub name_display : String,
    pub path : String,
    pub user_type : String,
    pub user_account : String,
    pub image_path : String,
}

/// Info: Users.
/// 
/// # Created By
/// - `vmm.map_user()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapUserEntry {
    pub user : String,
    pub sid : String,
    pub va_reg_hive : u64,
}

/// Info: Virtual Machines (VMs).
/// 
/// # Created By
/// - `vmm.map_virtual_machine()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmMapVirtualMachineEntry {
    h_vmm : usize,
    h_vm : usize,
    pub name : String,
    pub gpa_max : u64,
    pub tp_vm : u32,
    pub is_active : bool,
    pub is_readonly : bool,
    pub is_physicalonly : bool,
    pub partition_id : u32,
    pub guest_os_version_build : u32,
    pub guest_tp_system : u32,
    pub parent_mount_id : u32,
    pub vmmem_pid : u32,
}

/// VFS (Virtual File System) entry information - file or directory.
/// 
/// # Created By
/// - `vmm.vfs_list()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmVfsEntry {
    /// Name of the file or directory.
    pub name : String,
    /// True if entry is a directory, False if entry is a file.
    pub is_directory : bool,
    /// File size if file.
    pub size : u64,
}

impl Vmm<'_> {
    /// <b>MemProcFS Initialization Function.</b>
    /// 
    /// The [`Vmm`] struct is the base of the MemProcFS API. All API accesses
    /// takes place from the [`Vmm`] struct and its sub-structs.
    /// 
    /// The [`Vmm`] struct acts as a wrapper around the native MemProcFS VMM API.
    /// 
    /// 
    /// # Arguments
    /// * `vmm_lib_path` - Full path to the native vmm library - i.e. `vmm.dll` or `vmm.so`.
    /// * `args` - MemProcFS command line arguments as a Vec<&str>.
    /// 
    /// MemProcFS command line argument documentation is found on the [MemProcFS wiki](https://github.com/ufrisk/MemProcFS/wiki/_CommandLine).
    /// 
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Initialize MemProcFS VMM on a Windows system parsing a
    /// // memory dump and virtual machines inside it.
    /// let args = ["-printf", "-v", "-waitinitialize", "-device", "C:\\Dumps\\mem.dmp"].to_vec();
    /// if let Ok(vmm) = Vmm::new("C:\\MemProcFS\\vmm.dll", &args) {
    ///     ...
    ///     // The underlying native vmm is automatically closed 
    ///     // when the vmm object goes out of scope.
    /// };
    /// ```
    /// 
    /// ```
    /// // Initialize MemProcFS VMM on a Linux system parsing live memory
    /// // retrieved from a PCILeech FPGA hardware device.
    /// let args = ["-device", "fpga"].to_vec();
    /// if let Ok(vmm) = Vmm::new("/home/user/memprocfs/vmm.so", &args) {
    ///     ...
    ///     // The underlying native vmm is automatically closed 
    ///     // when the vmm object goes out of scope.
    /// };
    /// ```
    pub fn new<'a>(vmm_lib_path : &str, args: &Vec<&str>) -> ResultEx<Vmm<'a>> {
        return crate::impl_new(vmm_lib_path, 0, args);
    }

    /// Initialize MemProcFS from a host VMM and a child VM.
    /// 
    /// Initialize a MemProcFS VMM object representing a child virtual machine (VM).
    /// 
    /// # Arguments
    /// * `vmm_parent` - The host (parent) [`Vmm`].
    /// * `vm_entry` - The [`VmmMapVirtualMachineEntry`] to initialize as a [`Vmm`].
    /// 
    /// # Examples
    /// ```
    /// if let Ok(virtualmachine_all) = vmm.map_virtual_machine() {
    ///     for virtualmachine in &*virtualmachine_all {
    ///         println!("{virtualmachine}");
    ///         if virtualmachine.is_active {
    ///             // for active vms it's possible to create a new vmm object for
    ///             // the vm. it's possible to treat this as any other vmm object
    ///             // to read memory, query processes etc.
    ///             let vmm_vm = match Vmm::new_from_virtual_machine(&vmm, &virtualmachine) {
    ///                 Err(_) => continue,
    ///                 Ok(r) => r,
    ///             };
    ///             let max_addr = vmm_vm.get_config(CONFIG_OPT_CORE_MAX_NATIVE_ADDRESS).unwrap_or(0);
    ///             println!("vm max native address: {:#x}", max_addr);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn new_from_virtual_machine<'a>(vmm_parent : &'a Vmm, vm_entry : &VmmMapVirtualMachineEntry) -> ResultEx<Vmm<'a>> {
        return impl_new_from_virtual_machine(vmm_parent, vm_entry);
    }

    /// Retrieve a single process by PID.
    /// 
    /// # Arguments
    /// * `pid` - Process id (PID) of the process to retrieve.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(process) = vmm.process_from_pid(4) {
    ///     println!("{}", process);    
    /// }
    /// ```
    pub fn process_from_pid(&self, pid : u32) -> ResultEx<VmmProcess> {
        return self.impl_process_from_pid(pid);
    }

    /// Retrieve a single process by name.
    /// 
    /// If multiple processes have the same name the first process located by
    /// MemProcFS will be returned. If it is important to fetch the correct
    /// process retrieve the process list from `vmm.list()` and iterate.
    /// 
    /// # Arguments
    /// * `process_name` - Name of the process to retrieve.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(process) = vmm.process_from_name("System") {
    ///     println!("{}", process);    
    /// }
    /// ```
    pub fn process_from_name(&self, process_name : &str) -> ResultEx<VmmProcess> {
        return self.impl_process_from_name(process_name);
    }

    /// Retrieve all processes.
    /// 
    /// # Examples
    /// ```
    /// // Retrieve all processes (as a Vec).
    /// process_all = vmm.process_list()?
    /// for process in &*process_all {
    ///     println!("{process} ");
    /// }
    /// ```
    pub fn process_list(&self) -> ResultEx<Vec<VmmProcess>> {
        return self.impl_process_list();
    }

    /// Retrieve all processes as a map.
    /// 
    /// K: PID,
    /// V: VmmProcess
    /// 
    /// # Examples
    /// ```
    ///  // Retrieve all processes as (a HashMap).
    /// process_all = vmm.process_map()?;
    /// for process in process_all {
    ///     println!("<{},{}> ", process.0, process.1);
    /// }
    /// ```
    pub fn process_map(&self) -> ResultEx<HashMap<u32, VmmProcess>> {
        return Ok(self.impl_process_list()?.into_iter().map(|s| (s.pid, s)).collect());
    }

    /// Get a numeric configuration value.
    /// 
    /// # Arguments
    /// * `config_id` - As specified by a `CONFIG_OPT_*` constant marked as `Get`. (Optionally or'ed | with process pid for select options).
    /// 
    /// # Examples
    /// ```
    /// println!("max addr: {:#x}", vmm.get_config(CONFIG_OPT_CORE_MAX_NATIVE_ADDRESS).unwrap_or(0));
    /// ```
    pub fn get_config(&self, config_id : u64) -> ResultEx<u64> {
        return self.impl_get_config(config_id);
    }

    /// Set a numeric configuration value.
    /// 
    /// # Arguments
    /// * `config_id` - As specified by a `CONFIG_OPT_*` constant marked as `Set`. (Optionally or'ed | with process pid for select options).
    /// * `config_value` - The config value to set.
    /// 
    /// # Examples
    /// ```
    /// // The below force MemProcFS to undertake a full refresh - refresing
    /// // processes, memory and other general data structures completely.
    /// let _r = vmm.set_config(CONFIG_OPT_REFRESH_ALL, 1);
    /// ```
    pub fn set_config(&self, config_id : u64, config_value : u64) -> ResultEx<()> {
        return self.impl_set_config(config_id, config_value);
    }

    /// Retrieve the kernel convenience struct.
    /// 
    /// The kernel struct provides easy access to kernel build number,
    /// the system process (pid 4) and kernel (nt) debug symbols.
    /// 
    /// # Examples
    /// ```
    /// // Retrieve and print the kernel build number.
    /// println!("{}", vmm.kernel().build());
    /// ```
    pub fn kernel(&self) -> VmmKernel {
        return VmmKernel { vmm : &self };
    }

    /// Log a message to the MemProcFS logging system.
    /// 
    /// # Arguments
    /// * `log_level`
    /// * `log_message`
    /// 
    /// # Examples
    /// ```
    /// vmm.log(&VmmLogLevel::_1Critical, "Test Message Critical!");
    /// ```
    pub fn log(&self, log_level : &VmmLogLevel, log_message : &str) {
        self.impl_log(VMMDLL_MID_RUST, log_level, log_message);
    }

    /// Retrieve the physical memory range info map.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(memory_range_all) = vmm.map_memory() {
    ///     for memory_range in &*memory_range_all {
    ///         println!("{memory_range} \t pa={:x} cb={:x}", memory_range.pa, memory_range.cb);
    ///     }
    /// }
    /// ```
    pub fn map_memory(&self) -> ResultEx<Vec<VmmMapMemoryEntry>> {
        return self.impl_map_memory();
    }

    /// Retrieve the network connection info map.
    /// 
    /// # Examples
    /// ```
    /// let net_all vmm.map_net()?;
    /// for net in &*net_all {
    ///     println!("{net}");
    /// }
    /// ```
    pub fn map_net(&self) -> ResultEx<Vec<VmmMapNetEntry>> {
        return self.impl_map_net();
    }

    /// Retrieve the page frame number (PFN) info map.
    /// 
    /// # Arguments
    /// * `pfns` - The PFNs to retrieve.
    /// * `is_extended` - Retrieve extended information (more resource intense).
    /// 
    /// # Examples
    /// ```
    /// let pfns: Vec<u32> = (1..=10).collect();
    /// if let Ok(pfn_all) = vmm.map_pfn(&pfns, true) {
    ///     for pfn in &*pfn_all {
    ///         println!("{pfn} \t location={} tp_ex={} pid={:x} va={:x} color={}",
    ///                  pfn.location, pfn.tp_ex, pfn.pid, pfn.va, pfn.color);
    ///     }
    /// }
    /// ```
    pub fn map_pfn(&self, pfns : &Vec<u32>, is_extended : bool) -> ResultEx<Vec<VmmMapPfnEntry>> {
        return self.impl_map_pfn(pfns, is_extended);
    }

    /// Retrieve the kernel pool allocation info map.
    /// 
    /// # Arguments
    /// * `is_bigpool_only` - Retrieve only entries from the big pool (faster).
    /// 
    /// # Examples
    /// ```
    /// if let Ok(pool_all) = vmm.map_pool(false) {
    ///     println!("Number of pool allocations: {}.", pool_all.len());
    ///     let pool_proc_all : Vec<&VmmMapPoolEntry> =
    ///             pool_all.iter().filter(|e| e.tag == 0x636f7250 /* 'Proc' backwards */).collect();
    ///     println!("Number of pool 'Proc' allocations: {}.", pool_all.len());
    ///     for pool_proc in &*pool_proc_all {
    ///         print!("{pool_proc} ");
    ///     }
    ///     println!("");
    /// }
    /// ```
    pub fn map_pool(&self, is_bigpool_only : bool) -> ResultEx<Vec<VmmMapPoolEntry>> {
        return self.impl_map_pool(is_bigpool_only);
    }

    /// Retrieve the servives info map.
    /// 
    /// # Examples
    /// ```
    /// let service_all = vmm.map_service()?;
    /// for service in &*service_all {
    ///     println!("{service} ");
    /// }
    /// ```
    pub fn map_service(&self) -> ResultEx<Vec<VmmMapServiceEntry>> {
        return self.impl_map_service();
    }

    /// Retrieve the user map.
    /// 
    /// # Examples
    /// ```
    /// let user_all = vmm.map_user()?;
    /// for user in &*user_all {
    ///     println!("{:x}:: {} :: {} :: {user}", user.va_reg_hive, user.sid, user.user);
    /// }
    /// ```
    pub fn map_user(&self) -> ResultEx<Vec<VmmMapUserEntry>> {
        return self.impl_map_user();
    }

    /// Retrieve the virtual machines info map.
    /// 
    /// # Examples
    /// ```
    /// let virtualmachine_all = vmm.map_virtual_machine()?
    /// for virtualmachine in &*virtualmachine_all {
    ///     println!("{virtualmachine}");
    ///     if virtualmachine.is_active {
    ///         // for active vms it's possible to create a new vmm object for
    ///         // the vm. it's possible to treat this as any other vmm object
    ///         // to read memory, query processes etc.
    ///         let vmm_vm = match Vmm::new_from_virtual_machine(&vmm, &virtualmachine) {
    ///             Err(_) => continue,
    ///             Ok(r) => r,
    ///         };
    ///         println!("vm max native address: {:#x} -> {:#x}",
    ///                  CONFIG_OPT_CORE_MAX_NATIVE_ADDRESS,
    ///                  vmm_vm.get_config(CONFIG_OPT_CORE_MAX_NATIVE_ADDRESS).unwrap_or(0));
    ///     }
    /// }
    /// ```
    pub fn map_virtual_machine(&self) -> ResultEx<Vec<VmmMapVirtualMachineEntry>> {
        return self.impl_map_virtual_machine();
    }

    /// Read a contigious physical memory chunk.
    /// 
    /// The physical memory is read without any special flags. The whole chunk
    /// must be read successfully for the method to succeed.
    /// 
    /// If deseriable to provide flags modifying the behavior (such as skipping
    /// the built-in data cache or slower paging access) use the method
    /// `mem_read_ex()` instead.
    /// 
    /// Reading many memory chunks individually may be slow, especially if
    /// reading takes place using hardware FPGA devices. In that case it's
    /// better to use the `mem_scatter()` functionality for better performance.
    /// 
    /// 
    /// # Arguments
    /// * `pa` - Physical address to start reading from.
    /// * `size` - Number of bytes to read.
    /// 
    /// # Examples
    /// ```
    /// // Read 0x100 bytes of data starting at address 0x1000.
    /// // Example assumes: use pretty_hex::*;
    /// if let Ok(data_read) = vmm.mem_read(0x1000, 0x100) {
    ///     println!("{:?}", data_read.hex_dump());
    /// }
    /// ```
    pub fn mem_read(&self, pa : u64, size : usize) -> ResultEx<Vec<u8>> {
        return self.impl_mem_read(u32::MAX, pa, size, 0);
    }

    /// Read a contigious physical memory chunk with flags.
    /// 
    /// Flags are constants named `FLAG_*`
    /// 
    /// Reading many memory chunks individually may be slow, especially if
    /// reading takes place using hardware FPGA devices. In that case it's
    /// better to use the `mem_scatter()` functionality for better performance.
    /// 
    /// 
    /// # Arguments
    /// * `pa` - Physical address to start reading from.
    /// * `size` - Number of bytes to read.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// // Read 0x100 bytes of data starting at address 0x1000.
    /// // Force reading the underlying memory device (skip data cache) and
    /// // Zero-Pad if parts of the memory read fail instead of failing.
    /// // Example assumes: use pretty_hex::*;
    /// if let Ok(data_read) = vmm.mem_read_ex(0x1000, 0x100, FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL) {
    ///     println!("{:?}", data_read.hex_dump());
    /// }
    /// ```
    pub fn mem_read_ex(&self, pa : u64, size : usize, flags : u64) -> ResultEx<Vec<u8>> {
        return self.impl_mem_read(u32::MAX, pa, size, flags);
    }

    /// Read a contigious physical memory chunk with flags as a type/struct.
    /// 
    /// Flags are constants named `FLAG_*`
    /// 
    /// Reading many memory chunks individually may be slow, especially if
    /// reading takes place using hardware FPGA devices. In that case it's
    /// better to use the `mem_scatter()` functionality for better performance.
    /// 
    /// 
    /// # Arguments
    /// * `pa` - Physical address to start reading from.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// // Read the C-struct IMAGE_DOS_HEADER from memory.
    /// // Force reading the underlying memory device (skip data cache).
    /// #[repr(C)]
    /// struct IMAGE_DOS_HEADER {
    ///     e_magic : u16,
    /// 	...
    ///     e_lfanew : u32,
    /// }
    /// if let Ok(doshdr) = vmm.mem_read_as::<IMAGE_DOS_HEADER>(pa_kernel32, FLAG_NOCACHE) {
    ///     println!("e_magic:  {:x}", doshdr.e_magic);
    ///     println!("e_lfanew: {:x}", doshdr.e_lfanew);
    /// }
    /// ```
    pub fn mem_read_as<T>(&self, pa : u64, flags : u64) -> ResultEx<T> {
        return self.impl_mem_read_as(u32::MAX, pa, flags);
    }

    /// Create a scatter memory object for efficient physical memory reads.
    /// 
    /// Check out the [`VmmScatterMemory`] struct for more detailed information.
    /// 
    /// # Arguments
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// let mem_scatter_physical = vmm.mem_scatter(FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL)?;
    /// ```
    pub fn mem_scatter(&self, flags : u64) -> ResultEx<VmmScatterMemory> {
        return self.impl_mem_scatter(u32::MAX, flags);
    }

    /// Write physical memory.
    /// 
    /// The write is a best effort. Even of the write should fail it's not
    /// certain that an error will be returned. To be absolutely certain that
    /// a write has taken place follow up with a read.
    /// 
    /// # Arguments
    /// * `pa` - Physical address to start writing from.
    /// * `data` - Byte data to write.
    /// 
    /// # Examples
    /// ```
    /// let data_to_write = [0x56u8, 0x4d, 0x4d, 0x52, 0x55, 0x53, 0x54].to_vec();
    /// let _r = vmm.mem_write(0x1000, &data_to_write);
    /// ```
    pub fn mem_write(&self, pa : u64, data : &Vec<u8>) -> ResultEx<()> {
        return self.impl_mem_write(u32::MAX, pa, data);
    }

    /// Write a type/struct to physical memory.
    /// 
    /// The write is a best effort. Even of the write should fail it's not
    /// certain that an error will be returned. To be absolutely certain that
    /// a write has taken place follow up with a read.
    /// 
    /// # Arguments
    /// * `pa` - Pnhysical address to start writing from.
    /// * `data` - Data to write. In case of a struct repr(C) is recommended.
    /// 
    /// # Examples
    /// ```
    /// let data_to_write = [0x56, 0x4d, 0x4d, 0x52, 0x55, 0x53, 0x54];
    /// let _r = vmm.mem_write_as(0x1000, &data_to_write);
    /// ```
    pub fn mem_write_as<T>(&self, pa : u64, data : &T) -> ResultEx<()> {
        return self.impl_mem_write_as(u32::MAX, pa, data);
    }

    /// List a VFS (Virtual File System) directory.
    /// 
    /// Returns a result containing the individual directory entries -
    /// which may be files or directories.
    /// 
    /// # Arguments
    /// * `path` - VFS path to list directory contents in. Ex: /sys/
    /// 
    /// # Examples
    /// ```
    /// let vfs_list_path = "/sys/";
    /// if let Ok(vfs_all) = vmm.vfs_list(vfs_list_path) {
    ///     println!("VFS directory listing for directory: {vfs_list_path}");
    ///     println!("Number of file/directory entries: {}.", vfs_all.len());
    ///     for vfs in &*vfs_all {
    ///         println!("{vfs}");
    ///     }
    /// }
    /// ```
    pub fn vfs_list(&self, path : &str) -> ResultEx<Vec<VmmVfsEntry>> {
        return self.impl_vfs_list(path);
    }

    /// Read a VFS (Virtual File System) file.
    /// 
    /// The read contents are returned as a Vec containing the byte results.
    /// If the end of the file is reached the number of read bytes may be
    /// shorter than the requested read size.
    /// 
    /// # Arguments
    /// * `filename` - Full vfs path of the file to read. Ex: /sys/version.txt
    /// * `size` - Number of bytes to read.
    /// * `offset` - File offset.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(vfs_file_data) = vmm.vfs_read("/sys/memory/physmemmap.txt", 0x2000, 0) {
    ///     println!("Bytes read from file '/sys/memory/physmemmap.txt': {}.", vfs_file_data.len());
    ///     println!("{:?}", vfs_file_data.hex_dump());
    /// }
    /// ```
    pub fn vfs_read(&self, filename : &str, size : u32, offset : u64) -> ResultEx<Vec<u8>> {
        return self.impl_vfs_read(filename, size, offset);
    }

    /// Write a VFS (Virtual File System) file.
    /// 
    /// Writes are undertaken on a best-effort basis. Writing to read-only
    /// files will have no meaning. Writing to memory may or may not be
    /// possible depending on various factors. If important, it's recommended
    /// to verify the `vfs_write()` with a `vfs_read()`.
    /// 
    /// # Arguments
    /// * `filename` - Full VFS path of the file to write. Ex: /conf/config_printf_enable.txt
    /// * `data` - Byte data to write.
    /// * `offset` - File offset.
    /// 
    /// # Examples
    /// ```
    /// let vfs_write_data = vec![1u8; 1];
    /// vmm.vfs_write("/conf/config_process_show_terminated.txt", vfs_write_data, 0);
    /// ```
    pub fn vfs_write(&self, filename : &str, data : Vec<u8>, offset : u64) {
        return self.impl_vfs_write(filename, data, offset);
    }

    /// Retrieve all registry hives.
    /// 
    /// # Examples
    /// ```
    /// let hive_all = vmm.reg_hive_list()?;
    /// for hive in hive_all {
    ///     println!("{hive} size={} path={}", hive.size, hive.path);
    /// }
    /// ```
    pub fn reg_hive_list(&self) -> ResultEx<Vec<VmmRegHive>> {
        return self.impl_reg_hive_list();
    }

    /// Retrieve a registry key by its path.
    /// 
    /// Registry keys may be addressed either by its full path or by hive address
    /// and hive path. Both addressing modes are shown in the examples below.
    /// Registry keys are case sensitive.
    /// 
    /// Check out the [`VmmRegKey`] struct for more detailed information.
    /// 
    /// # Examples
    /// ```
    /// // Retrieve a regkey by full path.
    /// let regkey = vmm.reg_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run")?
    /// println!("{regkey");
    /// ```
    /// 
    /// ```
    /// // Retrieve a regkey by hive path.
    /// // (SOFTWARE hive example address: 0xffffba061a908000).
    /// let regkey = vmm.reg_key("0xffffba061a908000\\ROOT\\Microsoft\\Windows\\CurrentVersion\\Run")?
    /// println!("{regkey");
    /// ```
    pub fn reg_key(&self, path : &str) -> ResultEx<VmmRegKey> {
        return self.impl_reg_key(path);
    }

    /// Retrieve a registry value by its path.
    /// 
    /// Registry values may be addressed either by its full path or by hive
    /// address and hive path. Both addressing modes are shown in the examples
    /// below. Registry keys are case sensitive.
    /// 
    /// Check out the [`VmmRegValue`] struct for more detailed information.
    /// 
    /// # Examples
    /// ```
    /// // Retrieve a regvalue by full path.
    /// let regpath = "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\ProgramFilesDir";
    /// let regvalue = vmm.reg_key(regpath)?
    /// println!("{regkey");
    /// ```
    /// 
    /// ```
    /// // Retrieve a regvalue by hive path.
    /// // (SOFTWARE hive example address: 0xffffba061a908000).
    /// regpath = "0xffffba061a908000\\ROOT\\Microsoft\\Windows\\CurrentVersion\\ProgramFilesDir";
    /// let regvalue = vmm.reg_key(regpath)?
    /// println!("{regkey");
    /// ```
    pub fn reg_value(&self, path : &str) -> ResultEx<VmmRegValue> {
        return self.impl_reg_value(path);
    }

    /// Retrieve a search struct for a physical memory search.
    /// 
    /// NB! This does not start the actual search yet. 
    /// 
    /// Check out the [`VmmRegValue`] struct for more detailed information.
    /// 
    /// 
    /// # Arguments
    /// * `addr_min` - Start search at this physical address.
    /// * `addr_max` - End the search at this physical address. 0 is interpreted as u64::MAX.
    /// * `num_results_max` - Max number of search hits to search for. Max allowed value is 0x10000.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// 
    /// # Examples
    /// ```
    /// // Retrieve a VmmSearch for the entire physical memory.
    /// let search = vmm.search(0, 0, 0x10000, 0)?
    /// ```
    /// 
    /// ```
    /// // Retrieve a VmmSearch for physical memory between 4GB and 8GB.
    /// // Also stop at first search hit.
    /// let search = vmm.search(0x100000000, 0x200000000, 1, 0)?
    /// ```
    pub fn search(&self, addr_min : u64, addr_max : u64, num_results_max : u32, flags : u64) -> ResultEx<VmmSearch> {
        return VmmSearch::impl_new(&self, u32::MAX, addr_min, addr_max, num_results_max, flags);
    }
}

impl VmmMapPoolEntry {
    /// Retrieve the pool entry tag String.
    pub fn tag_to_string(&self) -> String {
        let tag_chars = [((self.tag >> 0) & 0xff) as u8, ((self.tag >> 8) & 0xff) as u8, ((self.tag >> 16) & 0xff) as u8, ((self.tag >> 24) & 0xff) as u8];
        return String::from_utf8_lossy(&tag_chars).to_string();
    }
}






/// Kernel information.
/// 
/// The kernel struct gives easy access to:
/// * The system process (pid 4).
/// * Kernel build number.
/// * Kernel debug symbols (nt).
/// 
/// 
/// # Created By
/// - `vmm.kernel()`
/// 
/// # Examples
/// ```
/// println!("{}", vmm.kernel().process());
/// println!("{}", vmm.kernel().build());
/// let kernel = vmm.kernel();
/// let pdb = kernel.pdb();
/// println!("{pdb}");
/// ```
#[derive(Debug)]
pub struct VmmKernel<'a> {
    vmm : &'a Vmm<'a>,
}

impl VmmKernel<'_> {
    /// Get the kernel build numer.
    /// 
    /// # Examples
    /// ```
    /// // Retrieve and print the kernel build number.
    /// println!("{}", vmm.kernel().build());
    /// ```
    pub fn build(&self) -> u32 {
        return self.vmm.get_config(CONFIG_OPT_WIN_VERSION_BUILD).unwrap_or_default().try_into().unwrap_or_default();
    }

    /// Get the System process (pid 4).
    /// 
    /// # Examples
    /// ```
    /// // Retrieve and print the kernel build number.
    /// let systemprocess = vmm.kernel().process();
    /// ```
    pub fn process(&self) -> VmmProcess {
        return VmmProcess { vmm : self.vmm, pid : 4 };
    }

    /// Get kernel debug information (nt).
    /// 
    /// For additional information about debug symbols check out the [`VmmPdb`] struct.
    /// 
    /// # Examples
    /// ```
    /// // Retrieve and print the kernel build number.
    /// let pdb_nt = vmm.kernel().pdb();
    /// ```
    pub fn pdb(&self) -> VmmPdb {
        return VmmPdb { vmm : self.vmm, module : String::from("nt") };
    }
}






/// Debug Symbol API.
/// 
/// The PDB sub-system requires that MemProcFS supporting DLLs/.SO's for
/// debugging and symbol server are put alongside `vmm.dll`. Also it's
/// recommended that the file `info.db` is put alongside `vmm.dll`.
/// 
/// 
/// # Created By
/// - `vmmprocess.pdb_from_module_address()`
/// - `vmm.kernel().pdb()`
/// 
/// # Examples
/// ```
/// // Retrieve the PDB struct associated with the kernel (nt).
/// let kernel = vmm.kernel();
/// let pdb = kernel.pdb();
/// ```
/// 
/// ```
/// // Retrieve the PDB struct associated with a process module.
/// let pdb = vmmprocess.pdb("ntdll.dll")?;
/// ```
#[derive(Debug)]
pub struct VmmPdb<'a> {
    vmm : &'a Vmm<'a>,
    pub module : String,
}

impl VmmPdb<'_> {
    /// Retrieve a symbol name and a displacement given a module offset or virtual address.
    /// 
    /// # Arguments
    /// * `va_or_offset` - Virtual address or offset from module base.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(r) = pdb.symbol_name_from_address(va_symbol) {
    ///     println!("va_o: {:x} name: '{}' displacement: {:x}", va_symbol, r.0, r.1);
    /// }
    /// ```
    pub fn symbol_name_from_address(&self, va_or_offset : u64) -> ResultEx<(String, u32)> {
        return self.impl_symbol_name_from_address(va_or_offset);
    }

    /// Lookup a symbol address given its name.
    /// 
    /// # Arguments
    /// * `symbol_name`
    /// 
    /// # Examples
    /// ```
    /// let va = pdb_nt.symbol_address_from_name("MiMapContiguousMemory")?;
    /// ```
    pub fn symbol_address_from_name(&self, symbol_name : &str) -> ResultEx<u64> {
        return self.impl_symbol_address_from_name(symbol_name);
    }

    /// Retrieve the size of a struct/type.
    /// 
    /// # Arguments
    /// * `type_name`
    /// 
    /// # Examples
    /// ```
    /// let size_eprocess = pdb_nt.type_size("_EPROCESS")?;
    /// ```
    pub fn type_size(&self, type_name : &str) -> ResultEx<u32> {
        return self.impl_type_size(type_name);
    }

    /// Retrieve offset of a struct child member.
    /// 
    /// # Arguments
    /// * `type_name`
    /// * `type_child_name`
    /// 
    /// # Examples
    /// ```
    /// let offet_vadroot = pdb_nt.type_child_offset("_EPROCESS", "VadRoot")?
    /// ```
    pub fn type_child_offset(&self, type_name : &str, type_child_name : &str) -> ResultEx<u32> {
        return self.impl_type_child_offset(type_name, type_child_name);
    }
}






/// Efficient Memory Reading API.
/// 
/// The Scatter Memory API allows reading several scattered memory regions at
/// the same time in one pass - greatly improving efficiency over multiple
/// normal memory reads.
/// 
/// The Rust Scatter API may be used in two different ways, both are displayed
/// below in the examples section.
/// 
/// 
/// # Created By
/// - `vmm.mem_scatter()`
/// - `vmmprocess.mem_scatter()`
/// 
/// # Example #1
/// ```
/// // Example: vmmprocess.mem_scatter() #1:
/// // This example will show how it's possible to use VmmScatterMemory to
/// // more efficiently read memory from the underlying device.
/// {
///     // Example: vmmprocess.mem_scatter():
///     // Retrieve a scatter memory read object that may be used to batch
///     // several reads/writes into one efficient call to the memory device.
///     println!("========================================");
///     println!("vmmprocess.mem_scatter() #1:");
///     let mem_scatter = vmmprocess.mem_scatter(FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL)?;
///     println!("mem_scatter = {mem_scatter}");
///     // Prepare three memory ranges to read.
///     let _r = mem_scatter.prepare(kernel32.va_base + 0x0000, 0x100);
///     let _r = mem_scatter.prepare(kernel32.va_base + 0x1000, 0x100);
///     let _r = mem_scatter.prepare(kernel32.va_base + 0x2000, 0x100);
///     // Perform the actual read (and writes) by calling the execute() function.
///     let _r = mem_scatter.execute();
///     // Fetch data read. It's possible (but wasteful) to read less data than was prepared.
///     if let Ok(data_read) = mem_scatter.read(kernel32.va_base + 0x0000, 0x80) {
///         println!("memory range: va={:x} cb={:x} cb_read={:x}", kernel32.va_base + 0x0000, 0x80, data_read.len());
///         println!("{:?}", data_read.hex_dump());
///         println!("-----------------------");
///     }
///     if let Ok(data_read) = mem_scatter.read(kernel32.va_base + 0x1000, 0x100) {
///         println!("memory range: va={:x} cb={:x} cb_read={:x}", kernel32.va_base + 0x1000, 0x100, data_read.len());
///         println!("{:?}", data_read.hex_dump());
///         println!("-----------------------");
///     }
///     // It's possible to do a re-read of the ranges by calling execute again!
///     let _r = mem_scatter.execute();
///     if let Ok(data_read) = mem_scatter.read(kernel32.va_base + 0x0000, 0x80) {
///         println!("memory range: va={:x} cb={:x} cb_read={:x}", kernel32.va_base + 0x0000, 0x80, data_read.len());
///         println!("{:?}", data_read.hex_dump());
///         println!("-----------------------");
///     }
///     // It's also possible to clear the VmmScatterMemory to start anew.
///     // Clearing is slightly more efficient than creating a new object.
///     // let _r = mem_scatter.clear();
/// 
///     // NB! the VmmScatterMemory struct will be automatically free'd
///     //     on the native backend when it goes out of scope.
/// }
/// ```
/// 
/// # Example #2
/// ```
/// // Example: vmmprocess.mem_scatter() #2:
/// // This example demo how it's possible to use the prepare_ex function
/// // which will populate the prepared data regions automatically when the
/// // VmmScatterMemory is dropped.
/// // It's not recommended to mix the #1 and #2 syntaxes.
/// {
///     // memory ranges to read are tuples:
///     // .0 = the virtual address to read.
///     // .1 = vector of u8 which memory should be read into.
///     // .2 = u32 receiving the bytes successfully read data.
///     let mut memory_range_1 = (kernel32.va_base + 0x0000, vec![0u8; 0x100], 0u32);
///     let mut memory_range_2 = (kernel32.va_base + 0x1000, vec![0u8; 0x100], 0u32);
///     let mut memory_range_3 = (kernel32.va_base + 0x2000, vec![0u8; 0x100], 0u32);
///     // Feed the ranges into a mutable VmmScatterMemory inside a
///     // separate scope. The actual memory read will take place when
///     // the VmmScatterMemory goes out of scope and are dropped.
///     println!("========================================");
///     println!("vmmprocess.mem_scatter() #2:");
///     if let Ok(mut mem_scatter) = vmmprocess.mem_scatter(FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL) {
///         let _r = mem_scatter.prepare_ex(&mut memory_range_1);
///         let _r = mem_scatter.prepare_ex(&mut memory_range_2);
///         let _r = mem_scatter.prepare_ex(&mut memory_range_3);
///     }
///     // Results should now be available in the memory ranges if the read
///     // was successful. Note that there is no guarantee that memory is
///     // read - make sure to check the .2 item - number of bytes read.
///     for memory_range in [memory_range_1, memory_range_2, memory_range_3] {
///         println!("memory range: va={:x} cb={:x} cb_read={:x}", memory_range.0, memory_range.1.len(), memory_range.2);
///         println!("{:?}", memory_range.1.hex_dump());
///         println!("-----------------------");
///     }
/// }
/// ```
#[derive(Debug)]
pub struct VmmScatterMemory<'a> {
    vmm : &'a Vmm<'a>,
    hs : usize,
    pid : u32,
    flags : u32,
    is_scatter_ex : bool,
}

impl <'a> VmmScatterMemory<'a> {
    /// Prepare a memory range for reading according to method #2.
    /// 
    /// Once the `mem_scatter.execute()` call has been made the memory
    /// read should (if successful) be found in the prepared tuple.
    /// 
    /// See the [`VmmScatterMemory`] struct for an example.
    /// 
    /// # Arguments
    /// * `data_to_read` - Tuple with data to prepare as below:
    ///   * `data_to_read.0` - Address to start read from.
    ///   * `data_to_read.1` - Byte Vec with space to fill with read data on success.
    ///   * `data_to_read.2` - Bytes actually read on `mem_scatter.execute()` call. Should be zero at call to `mem_scatter.prepare_ex()`.
    pub fn prepare_ex(&mut self, data_to_read : &'a mut (u64, Vec<u8>, u32)) -> ResultEx<()> {
        return self.impl_prepare_ex(data_to_read);
    }

    /// Prepare a memory range for reading according to method #2.
    /// 
    /// Once the `mem_scatter.execute()` call has been made the memory
    /// read should (if successful) be found in the prepared tuple.
    /// 
    /// See the [`VmmScatterMemory`] struct for an example.
    /// 
    /// # Arguments
    /// * `data_to_read` - Tuple with data to prepare as below:
    ///   * `data_to_read.0` - Address to start read from.
    ///   * `data_to_read.1` - Generic Type/Struct to fill with read data on success.
    ///   * `data_to_read.2` - Bytes actually read on `mem_scatter.execute()` call. Should be zero at call to `mem_scatter.prepare_ex()`.
    pub fn prepare_ex_as<T>(&mut self, data_to_read : &'a mut (u64, T, u32)) -> ResultEx<()> {
        return self.impl_prepare_ex_as(data_to_read);
    }
}

impl VmmScatterMemory<'_> {
    /// Prepare a memory range for reading according to method #1.
    /// 
    /// Once the `mem_scatter.execute()` call has been made it's possible
    /// to read the memory by calling `mem_scatter.read()`.
    /// 
    /// See the [`VmmScatterMemory`] struct for an example.
    /// 
    /// # Arguments
    /// * `va` - Address to prepare to read from.
    /// * `size` - Number of bytes to read.
    pub fn prepare(&self, va : u64, size : usize) -> ResultEx<()> {
        return self.impl_prepare(va, size);
    }

    /// Prepare a memory range for reading according to method #1.
    /// 
    /// Once the `mem_scatter.execute()` call has been made it's possible
    /// to read the memory by calling `mem_scatter.read()`.
    /// 
    /// See the [`VmmScatterMemory`] struct for an example.
    /// 
    /// # Arguments
    /// * `va` - Address to prepare to read from.
    pub fn prepare_as<T>(&self, va : u64) -> ResultEx<()> {
        return self.impl_prepare(va, std::mem::size_of::<T>());
    }

    /// Prepare a memory range for writing.
    /// 
    /// Writing takes place on the call to `mem_scatter.execute()`.
    /// 
    /// # Arguments
    /// * `va` - Address to prepare to write to.
    /// * `data` - Data to write.
    pub fn prepare_write(&self, va : u64, data : &Vec<u8>) -> ResultEx<()> {
        return self.impl_prepare_write(va, data);
    }

    /// Prepare a memory range for writing.
    /// 
    /// Writing takes place on the call to `mem_scatter.execute()`.
    /// 
    /// # Arguments
    /// * `va` - Address to prepare to write to.
    /// * `data` - Data to write. In case of a struct repr(C) is recommended.
    pub fn prepare_write_as<T>(&self, va : u64, data : &T) -> ResultEx<()> {
        return self.impl_prepare_write_as(va, data);
    }

    /// Execute the scatter call to the underlying memory device.
    /// 
    /// This function takes care of all reading and writing. After
    /// this function is called it's possible to read memory, or if
    /// approach #2 is used the memory should already be read into
    /// buffers prepared with the call to `mem_scatter.prepare_ex()`.
    pub fn execute(&self) -> ResultEx<()> {
        return self.impl_execute();
    }

    /// Read memory prepared after the `execute()` call.
    pub fn read(&self, va : u64, size : usize) -> ResultEx<Vec<u8>> {
        return self.impl_read(va, size);
    }

    /// Read memory prepared after the `execute()` call.
    pub fn read_as<T>(&self, va : u64) -> ResultEx<T> {
        return self.impl_read_as(va);
    }

    /// Clear the scatter memory for additional read/writes.
    pub fn clear(&self) -> ResultEx<()> {
        return self.impl_clear();
    }
}






/// <b>Process API Base Struct.</b>
/// 
/// The [`VmmProcess`] struct is the base of the per-process related
/// functionality of the MemProcFS API. The [`VmmProcess`] struct should
/// be considered a child to the main [`Vmm`] struct.
/// 
/// <b>Check out the example project for more detailed API usage and
/// additional examples!</b>
/// 
/// 
/// # Created By
/// - `vmm.process_from_pid()`
/// - `vmm.process_from_name()`
/// - `vmm.process_list()`
/// - `vmm.kernel().process()`
/// - `plugin sub-system`
/// 
/// 
/// # Examples
/// 
/// ```
/// // Retrieve all processes:
/// if let Ok(process_all) = vmm.process_list() {
///     for process in &*process_all {
///         print!("{process} ");
///     }
/// }
/// ```
/// 
/// ```
/// // Retrieve a process by its name. If more than one process share the
/// // same name the first found will be returned.
/// if let Ok(systemprocess) = vmm.process_from_name("System") {
///     print!("{process} ");
/// };
/// ```
/// 
/// ```
/// // Retrieve a process by its PID.
/// if let Ok(systemprocess) = vmm.process_from_pid(4) {
///     print!("{process} ");
/// };
/// ```
#[derive(Debug)]
pub struct VmmProcess<'a> {
    vmm : &'a Vmm<'a>,
    pub pid : u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmIntegrityLevelType {
    Unknown,
    Untrusted,
    Low,
    Medium,
    MediumPlus,
    High,
    System,
    Protected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmMemoryModelType {
    NA,
    X86,
    X86PAE,
    X64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmSystemType {
    UnknownPhysical,
    UnknownX64,
    WindowsX64,
    UnknownX86,
    WindowsX86,
}

/// Process Information.
/// 
/// # Created By
/// - `vmmprocess.info()`
/// 
/// # Examples
/// ```
/// // Retrieve the VmmProcess info struct from a process.
/// // It's better to retrieve this struct once and query its fields rather
/// // than calling `vmmprocess.info()` repetedly since there is a small
/// // native overhead doing so.
/// if let Ok(procinfo) = vmmprocess.info() {
///     println!("struct   -> {procinfo}");
///     println!("pid      -> {}", procinfo.pid);
///     println!("ppid     -> {}", procinfo.pid);
///     println!("peb      -> {:x}", procinfo.va_peb);
///     println!("eprocess -> {:x}", procinfo.va_eprocess);
///     println!("name     -> {}", procinfo.name);
///     println!("longname -> {}", procinfo.name_long);
///     println!("SID      -> {}", procinfo.sid);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessInfo {
    pub pid : u32,
    pub ppid : u32,
    pub name : String,
    pub name_long : String,
    pub tp_system : VmmSystemType,
    pub tp_memorymodel : VmmMemoryModelType,
    pub is_user_mode : bool,
    pub state : u32,
    pub pa_dtb : u64,
    pub pa_dtb_user : u64,
    pub va_eprocess : u64,
    pub va_peb : u64,
    pub is_wow64 : bool,
    pub va_peb32 : u32,
    pub session_id : u32,
    pub luid : u64,
    pub sid : String,
    pub integrity_level : VmmIntegrityLevelType,
}

/// Info: Process Module: PE data directories.
/// 
/// # Created By
/// - `vmmprocess.map_module_data_directory()`
/// 
/// # Examples
/// ```
/// if let Ok(data_directory_all) = vmmprocess.map_module_data_directory("kernel32.dll") {
///     println!("Number of module data directories: {}.", data_directory_all.len());
///     for data_directory in &*data_directory_all {
///         println!("{data_directory}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapDirectoryEntry {
    pub pid : u32,
    pub name : &'static str,
    pub virtual_address : u32,
    pub size : u32,
}

/// Info: Process Module: PE exported entries.
/// 
/// # Created By
/// - `vmmprocess.map_module_eat()`
/// 
/// # Examples
/// ```
/// if let Ok(eat_all) = vmmprocess.map_module_eat("kernel32.dll") {
///     println!("Number of module exported functions: {}.", eat_all.len());
///     for eat in &*eat_all {
///         println!("{eat} :: {}", eat.forwarded_function);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapEatEntry {
    pub pid : u32,
    pub va_function : u64,
    pub ordinal : u32,
    pub function : String,
    pub forwarded_function : String,
}

/// Info: Process: Handles.
/// 
/// # Created By
/// - `vmmprocess.map_handle()`
/// 
/// # Examples
/// ```
/// if let Ok(handle_all) = vmmprocess.map_handle() {
///     println!("Number of handle entries: {}.", handle_all.len());
///     for handle in &*handle_all {
///         println!("{handle}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapHandleEntry {
    pub pid : u32,
    pub va_object : u64,
    pub handle_id : u32,
    pub granted_access : u32,
    pub type_index : u32,
    pub handle_count : u64,
    pub pointer_count : u64,
    pub va_object_create_info : u64,
    pub va_security_descriptor : u64,
    pub handle_pid : u32,
    pub pool_tag : u32,
    pub info : String,
    pub tp : String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmProcessMapHeapType {
    NA,
    NtHeap,
    SegmentHeap,
}

/// Info: Process: Heaps.
/// 
/// # Created By
/// - `vmmprocess.map_heap()`
/// 
/// # Examples
/// ```
/// if let Ok(heap_all) = vmmprocess.map_heap() {
///     println!("Number of heap entries: {}.", heap_all.len());
///     for heap in &*heap_all {
///         println!("{heap}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapHeapEntry {
    pub pid : u32,
    pub tp : VmmProcessMapHeapType,
    pub is_32 : bool,
    pub index : u32,
    pub number : u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmProcessMapHeapAllocType {
    NA,
    NtHeap,
    NtLFH,
    NtLarge,
    NtNA,
    SegVS,
    SegLFH,
    SegLarge,
    SegNA,
}

/// Info: Process: Heap allocations.
/// 
/// # Created By
/// - `vmmprocess.map_heapalloc()`
/// 
/// # Examples
/// ```
/// if let Ok(heapalloc_all) = vmmprocess.map_heapalloc(0) {
///     println!("Number of allocated heap entries: {}.", heapalloc_all.len());
///     for heapalloc in &*heapalloc_all {
///         print!("{heapalloc} ");
///     }
///     println!("");
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapHeapAllocEntry {
    pub pid : u32,
    pub va : u64,
    pub size : u32,
    pub tp : VmmProcessMapHeapAllocType,
}

/// Info: Process Module: PE imported entries.
/// 
/// # Created By
/// - `vmmprocess.map_module_iat()`
/// 
/// # Examples
/// ```
/// if let Ok(iat_all) = vmmprocess.map_module_iat("kernel32.dll") {
///     println!("Number of module imported functions: {}.", iat_all.len());
///     for iat in &*iat_all {
///         println!("{iat}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapIatEntry {
    pub pid : u32,
    pub va_function : u64,
    pub function : String,
    pub module : String,
}

/// Info: Process: Modules (loaded DLLs) debug information.
/// 
/// # Created By
/// - `vmmprocess.map_module()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapModuleDebugEntry {
    pub pid : u32,
    pub age : u32,
    pub raw_guid : [u8; 16],
    pub guid : String,
    pub pdb_filename : String,
}

/// Info: Process: Modules (loaded DLLs) version information.
/// 
/// # Created By
/// - `vmmprocess.map_module()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapModuleVersionEntry {
    pub pid : u32,
    pub company_name : String,
    pub file_description : String,
    pub file_version : String,
    pub internal_name : String,
    pub legal_copyright : String,
    pub original_file_name : String,
    pub product_name : String,
    pub product_version : String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmProcessMapModuleType {
    Normal,
    Data,
    NotLinked,
    Injected,
}

/// Info: Process: Modules (loaded DLLs).
/// 
/// # Created By
/// - `vmmprocess.map_module()`
/// 
/// # Examples
/// ```
/// if let Ok(module_all) = vmmprocess.map_module(true, true) {
///     println!("Number of process modules: {}.", module_all.len());
///     for module in &*module_all {
///         println!("{module}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapModuleEntry {
    pub pid : u32,
    pub va_base : u64,
    pub va_entry : u64,
    pub image_size : u32,
    pub is_wow64 : bool,
    pub tp : VmmProcessMapModuleType,
    pub name : String,
    pub full_name : String,
    pub file_size_raw : u32,
    pub section_count : u32,
    pub eat_count : u32,
    pub iat_count : u32,
    pub debug_info : Option<VmmProcessMapModuleDebugEntry>,
    pub version_info : Option<VmmProcessMapModuleVersionEntry>,
}

/// Info: Process: PTE memory map entries.
/// 
/// # Created By
/// - `vmmprocess.map_pte()`
/// 
/// # Examples
/// ```
/// if let Ok(pte_all) = vmmprocess.map_pte(true) {
///     println!("Number of pte entries: {}.", pte_all.len());
///     for pte in &*pte_all {
///         let s = if pte.is_s { 's' } else { '-' };
///         let r = if pte.is_r { 'r' } else { '-' };
///         let w = if pte.is_w { 'w' } else { '-' };
///         let x = if pte.is_x { 'x' } else { '-' };
///         println!("{pte} :: {s}{r}{w}{x} :: {}", pte.info);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapPteEntry {
    pub pid : u32,
    pub va_base : u64,
    pub page_count : u64,
    pub page_software_count : u32,
    pub is_wow64 : bool,
    pub info : String,
    pub is_r : bool,
    pub is_w : bool,
    pub is_x : bool,
    pub is_s : bool,
}

/// Info: Process Module: PE sections.
/// 
/// # Created By
/// - `vmmprocess.map_module_section()`
/// 
/// # Examples
/// ```
/// if let Ok(section_all) = vmmprocess.map_module_section("kernel32.dll") {
///     println!("Number of module sections: {}.", section_all.len());
///     for section in &*section_all {
///         println!("{section}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessSectionEntry {
    pub pid : u32,
    pub index : u32,
    pub name : String,
    pub name_raw : [u8; 8],
    pub misc_virtual_size : u32,
    pub virtual_address : u32,
    pub size_of_raw_data : u32,
    pub pointer_to_raw_data : u32,
    pub pointer_to_relocations : u32,
    pub pointer_to_linenumbers : u32,
    pub number_of_relocations : u16,
    pub number_of_linenumbers : u16,
    pub characteristics : u32,
}

/// Info: Process: Threads.
/// 
/// # Created By
/// - `vmmprocess.map_thread()`
/// 
/// # Examples
/// ```
/// if let Ok(thread_all) = vmmprocess.map_thread() {
///     println!("Number of process threads: {}.", thread_all.len());
///     for thread in &*thread_all {
///         println!("{thread}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapThreadEntry {
    pub pid : u32,
    pub thread_id : u32,
    pub thread_pid : u32,
    pub exit_status : u32,
    pub state : u8,
    pub running : u8,
    pub priority : u8,
    pub priority_base : u8,
    pub va_ethread : u64,
    pub va_teb : u64,
    pub ft_create_time : u64,
    pub ft_exit_time : u64,
    pub va_start_address : u64,
    pub va_win32_start_address : u64,
    pub va_stack_user_base : u64,
    pub va_stack_user_limit : u64,
    pub va_stack_kernel_base : u64,
    pub va_stack_kernel_limit : u64,
    pub va_trap_frame : u64,
    pub va_rip : u64,
    pub va_rsp : u64,
    pub affinity : u64,
    pub user_time : u32,
    pub kernel_time : u32,
    pub suspend_count : u8,
    pub wait_reason : u8
}

/// Info: Process: Unloaded modules.
/// 
/// # Created By
/// - `vmmprocess.map_unloaded_module()`
/// 
/// # Examples
/// ```
/// if let Ok(unloaded_all) = vmmprocess.map_unloaded_module() {
///     println!("Number of process unloaded modules: {}.", unloaded_all.len());
///     for unloaded in &*unloaded_all {
///         println!("{unloaded}");
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapUnloadedModuleEntry {
    pub pid : u32,
    pub va_base : u64,
    pub image_size : u32,
    pub is_wow64 : bool,
    pub name : String,
    pub checksum : u32,         // user-mode only
    pub timedatestamp : u32,    // user-mode only
    pub ft_unload : u64,        // kernel-mode only
}

/// Info: Process: VAD (Virtual Address Descriptor) memory map entries.
/// 
/// # Created By
/// - `vmmprocess.map_vad()`
/// 
/// # Examples
/// ```
/// if let Ok(vad_all) = vmmprocess.map_vad(true) {
///     println!("Number of vad entries: {}.", vad_all.len());
///     for vad in &*vad_all {
///         println!("{vad} :: {}", vad.info);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapVadEntry {
    pub pid : u32,
    pub va_start : u64,
    pub va_end : u64,
    pub va_vad : u64,
    pub u0 : u32,
    pub u1 : u32,
    pub u2 : u32,
    pub commit_charge : u32,
    pub is_mem_commit : bool,
    pub cb_prototype_pte : u32,
    pub va_prototype_pte : u64,
    pub va_subsection : u64,
    pub va_file_object : u64,
    pub info : String,
    pub vadex_page_base : u32,
    pub vadex_page_count : u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmmProcessMapVadExType {
    NA,
    Hardware,
    Transition,
    Prototype,
    DemandZero,
    Compressed,
    Pagefile,
    File,
}

/// Info: Process: Extended VAD memory map entries.
/// 
/// # Created By
/// - `vmmprocess.map_vadex()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmProcessMapVadExEntry {
    pub pid : u32,
    pub tp : VmmProcessMapVadExType,
    pub i_pml : u32,
    pub va : u64,
    pub pa : u64,
    pub pte : u64,
    pub proto_tp : VmmProcessMapVadExType,
    pub proto_pa : u64,
    pub proto_pte : u64,
    pub va_vad_base : u64,
}

impl VmmProcess<'_> {
    /// Get the base virtual address for a loaded module.
    /// 
    /// # Arguments
    /// * `module_name`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(module_base_kernel32) = vmmprocess.get_module_base("kernel32.dll") {
    ///     println!("kernel32.dll -> {:x}", module_base_kernel32);
    /// }
    /// ```
    pub fn get_module_base(&self, module_name : &str) -> ResultEx<u64> {
        return self.impl_get_module_base(module_name);
    }

    /// Get the address of an exported function or symbol.
    /// 
    /// This is similar to the Windows function GetProcAddress.
    /// 
    /// # Arguments
    /// * `module_name`
    /// * `function_name`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(procaddress) = vmmprocess.get_proc_address("kernel32.dll", "GetProcAddress") {
    ///     println!("kernel32.dll!GetProcAddress -> {:x}", procaddress);
    /// }
    /// ```
    pub fn get_proc_address(&self, module_name : &str, function_name : &str) -> ResultEx<u64> {
        return self.impl_get_proc_address(module_name, function_name);
    }
    pub fn get_proc_address_pid(&self, pid: u32, module_name : &str, function_name : &str) -> ResultEx<u64> {
        return self.impl_get_proc_address_pid(pid, module_name, function_name);
    }

    /// Get the process path (retrieved fom kernel mode).
    /// 
    /// # Examples
    /// ```
    /// if let Ok(path) = vmmprocess.get_path_kernel() {
    ///     println!("-> {path}");
    /// }
    /// ```
    pub fn get_path_kernel(&self) -> ResultEx<String> {
        return self.impl_get_information_string(VMMDLL_PROCESS_INFORMATION_OPT_STRING_PATH_KERNEL);
    }

    /// Get the process path (retrieved from user-mode).
    /// 
    /// # Examples
    /// ```
    /// if let Ok(path) = vmmprocess.get_path_user() {
    ///     println!("-> {path}");
    /// }
    /// ```
    pub fn get_path_user(&self) -> ResultEx<String> {
        return self.impl_get_information_string(VMMDLL_PROCESS_INFORMATION_OPT_STRING_PATH_USER_IMAGE);
    }

    /// Get the process command line.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(s_cmdline) = vmmprocess.get_cmdline() {
    ///     println!("-> {s_cmdline}");
    /// }
    /// ```
    pub fn get_cmdline(&self) -> ResultEx<String> {
        return self.impl_get_information_string(VMMDLL_PROCESS_INFORMATION_OPT_STRING_CMDLINE);
    }

    /// Get process information - such as name, ppid, state, etc.
    /// 
    /// If retrieving multiple values from the [`VmmProcessInfo`] struct it's
    /// recommended to retrieve the info object once instead of repetedly
    /// calling the info() method.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(procinfo) = vmmprocess.info() {
    ///     println!("struct   -> {procinfo}");
    ///     println!("pid      -> {}", procinfo.pid);
    ///     println!("ppid     -> {}", procinfo.pid);
    ///     println!("peb      -> {:x}", procinfo.va_peb);
    ///     println!("eprocess -> {:x}", procinfo.va_eprocess);
    ///     println!("name     -> {}", procinfo.name);
    ///     println!("longname -> {}", procinfo.name_long);
    ///     println!("SID      -> {}", procinfo.sid);
    /// }
    /// ```
    pub fn info(&self) -> ResultEx<VmmProcessInfo> {
        return self.impl_info();
    }

    /// Retrieve the handles info map.
    /// 
    /// For additional information see the [`VmmProcessMapHandleEntry`] struct.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(handle_all) = vmmprocess.map_handle() {
    ///     println!("Number of handle entries: {}.", handle_all.len());
    ///     for handle in &*handle_all {
    ///         println!("{handle}");
    ///     }
    /// }
    /// ```
    pub fn map_handle(&self) -> ResultEx<Vec<VmmProcessMapHandleEntry>> {
        return self.impl_map_handle();
    }

    /// Retrieve the heaps info map.
    /// 
    /// For additional information see the [`VmmProcessMapHeapEntry`] struct.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(heap_all) = vmmprocess.map_heap() {
    ///     println!("Number of heap entries: {}.", heap_all.len());
    ///     for heap in &*heap_all {
    ///         println!("{heap}");
    ///     }
    /// }
    /// ```
    pub fn map_heap(&self) -> ResultEx<Vec<VmmProcessMapHeapEntry>> {
        return self.impl_map_heap();
    }

    /// Retrieve the heap allocations info map.
    /// 
    /// For additional information see the [`VmmProcessMapHeapAllocEntry`] struct.
    /// 
    /// # Arguments
    /// * `heap_number_or_address` - Heap number as given by [`VmmProcessMapHeapEntry`] or the heap base address.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(heapalloc_all) = vmmprocess.map_heapalloc(0) {
    ///     println!("Number of allocated heap entries: {}.", heapalloc_all.len());
    ///     for heapalloc in &*heapalloc_all {
    ///         print!("{heapalloc} ");
    ///     }
    ///     println!("");
    /// }
    /// ```
    pub fn map_heapalloc(&self, heap_number_or_address : u64) -> ResultEx<Vec<VmmProcessMapHeapAllocEntry>> {
        return self.impl_map_heapalloc(heap_number_or_address);
    }

    /// Retrieve the loaded modules map.
    /// 
    /// For additional information see the [`VmmProcessMapModuleEntry`] struct.
    /// 
    /// # Arguments
    /// * `is_info_debug` - Also retrieve debug information.
    /// * `is_info_version` - Also version information.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(module_all) = vmmprocess.map_module(true, true) {
    ///     println!("Number of process modules: {}.", module_all.len());
    ///     for module in &*module_all {
    ///         println!("{module}");
    ///     }
    /// }
    /// ```
    pub fn map_module(&self, is_info_debug : bool, is_info_version : bool) -> ResultEx<Vec<VmmProcessMapModuleEntry>> {
        return self.impl_map_module(is_info_debug, is_info_version);
    }

    /// Retrieve PE data directories associated with a module.
    /// 
    /// For additional information see the [`VmmProcessMapDirectoryEntry`] struct.
    /// 
    /// # Arguments
    /// * `module_name`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(data_directory_all) = vmmprocess.map_module_data_directory("kernel32.dll") {
    ///     println!("Number of module data directories: {}.", data_directory_all.len());
    ///     for data_directory in &*data_directory_all {
    ///         println!("{data_directory}");
    ///     }
    /// }
    /// ```
    pub fn map_module_data_directory(&self, module_name : &str) -> ResultEx<Vec<VmmProcessMapDirectoryEntry>> {
        return self.impl_map_module_data_directory(module_name);
    }

    /// Retrieve exported functions and symbols associated with a module.
    /// 
    /// For additional information see the [`VmmProcessMapEatEntry`] struct.
    /// 
    /// # Arguments
    /// * `module_name`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(eat_all) = vmmprocess.map_module_eat("kernel32.dll") {
    ///     println!("Number of module exported functions: {}.", eat_all.len());
    ///     for eat in &*eat_all {
    ///         println!("{eat} :: {}", eat.forwarded_function);
    ///     }
    /// }
    /// ```
    pub fn map_module_eat(&self, module_name : &str) -> ResultEx<Vec<VmmProcessMapEatEntry>> {
        return self.impl_map_module_eat(module_name);
    }

    /// Retrieve imported functions associated with a module.
    /// 
    /// For additional information see the [`VmmProcessMapIatEntry`] struct.
    /// 
    /// # Arguments
    /// * `module_name`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(iat_all) = vmmprocess.map_module_iat("kernel32.dll") {
    ///     println!("Number of module imported functions: {}.", iat_all.len());
    ///     for iat in &*iat_all {
    ///         println!("{iat}");
    ///     }
    /// }
    /// ```
    pub fn map_module_iat(&self, module_name : &str) -> ResultEx<Vec<VmmProcessMapIatEntry>> {
        return self.impl_map_module_iat(module_name);
    }

    /// Retrieve PE sections associated with a module.
    /// 
    /// For additional information see the [`VmmProcessSectionEntry`] struct.
    /// 
    /// # Arguments
    /// * `module_name`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(section_all) = vmmprocess.map_module_section("kernel32.dll") {
    ///     println!("Number of module sections: {}.", section_all.len());
    ///     for section in &*section_all {
    ///         println!("{section}");
    ///     }
    /// }
    /// ```
    pub fn map_module_section(&self, module_name : &str) -> ResultEx<Vec<VmmProcessSectionEntry>> {
        return self.impl_map_module_section(module_name);
    }

    /// Retrieve the PTE memory info map.
    /// 
    /// For additional information see the [`VmmProcessMapPteEntry`] struct.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(pte_all) = vmmprocess.map_pte(true) {
    ///     println!("Number of pte entries: {}.", pte_all.len());
    ///     for pte in &*pte_all {
    ///         let s = if pte.is_s { 's' } else { '-' };
    ///         let r = if pte.is_r { 'r' } else { '-' };
    ///         let w = if pte.is_w { 'w' } else { '-' };
    ///         let x = if pte.is_x { 'x' } else { '-' };
    ///         println!("{pte} :: {s}{r}{w}{x} :: {}", pte.info);
    ///     }
    /// }
    /// ```
    pub fn map_pte(&self, is_identify_modules : bool) -> ResultEx<Vec<VmmProcessMapPteEntry>> {
        return self.impl_map_pte(is_identify_modules);
    }

    /// Retrieve the thread info map.
    /// 
    /// For additional information see the [`VmmProcessMapThreadEntry`] struct.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(thread_all) = vmmprocess.map_thread() {
    ///     println!("Number of process threads: {}.", thread_all.len());
    ///     for thread in &*thread_all {
    ///         println!("{thread}");
    ///     }
    /// }
    /// ```
    pub fn map_thread(&self) -> ResultEx<Vec<VmmProcessMapThreadEntry>> {
        return self.impl_map_thread();
    }

    /// Retrieve the unloaded module info map.
    /// 
    /// For additional information see the [`VmmProcessMapUnloadedModuleEntry`] struct.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(unloaded_all) = vmmprocess.map_unloaded_module() {
    ///     println!("Number of process unloaded modules: {}.", unloaded_all.len());
    ///     for unloaded in &*unloaded_all {
    ///         println!("{unloaded}");
    ///     }
    /// }
    /// ```
    pub fn map_unloaded_module(&self) -> ResultEx<Vec<VmmProcessMapUnloadedModuleEntry>> {
        return self.impl_map_unloaded_module();
    }

    /// Retrieve the VAD (virtual address descriptor) memory info map.
    /// 
    /// For additional information see the [`VmmProcessMapVadEntry`] struct.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(vad_all) = vmmprocess.map_vad(true) {
    ///     println!("Number of vad entries: {}.", vad_all.len());
    ///     for vad in &*vad_all {
    ///         println!("{vad} :: {}", vad.info);
    ///     }
    /// }
    /// ```
    pub fn map_vad(&self, is_identify_modules : bool) -> ResultEx<Vec<VmmProcessMapVadEntry>> {
        return self.impl_map_vad(is_identify_modules);
    }

    /// Retrieve the extended VAD info map.
    /// 
    /// For additional information see the [`VmmProcessMapVadExEntry`] struct.
    pub fn map_vadex(&self, offset_pages : u32, count_pages : u32) -> ResultEx<Vec<VmmProcessMapVadExEntry>> {
        return self.impl_map_vadex(offset_pages, count_pages);
    }

    /// Read a contigious virtual memory chunk.
    /// 
    /// The virtual memory is read without any special flags. The whole chunk
    /// must be read successfully for the method to succeed.
    /// 
    /// If deseriable to provide flags modifying the behavior (such as skipping
    /// the built-in data cache or slower paging access) use the method
    /// `mem_read_ex()` instead.
    /// 
    /// Reading many memory chunks individually may be slow, especially if
    /// reading takes place using hardware FPGA devices. In that case it's
    /// better to use the `mem_scatter()` functionality for better performance.
    /// 
    /// 
    /// # Arguments
    /// * `va` - Virtual address to start reading from.
    /// * `size` - Number of bytes to read.
    /// 
    /// # Examples
    /// ```
    /// // Read 0x100 bytes of data from the base of kernel32.
    /// // Example assumes: use pretty_hex::*;
    /// if let Ok(data_read) = vmmprocess.mem_read(va_kernel32, 0x100) {
    ///     println!("{:?}", data_read.hex_dump());
    /// }
    /// ```
    pub fn mem_read(&self, va : u64, size : usize) -> ResultEx<Vec<u8>> {
        return self.vmm.impl_mem_read(self.pid, va, size, 0);
    }

    /// Read a contigious virtual memory chunk with flags.
    /// 
    /// Flags are constants named `FLAG_*`
    /// 
    /// Reading many memory chunks individually may be slow, especially if
    /// reading takes place using hardware FPGA devices. In that case it's
    /// better to use the `mem_scatter()` functionality for better performance.
    /// 
    /// 
    /// # Arguments
    /// * `va` - Virtual address to start reading from.
    /// * `size` - Number of bytes to read.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// // Read 0x100 bytes of data from the base of kernel32.
    /// // Force reading the underlying memory device (skip data cache) and
    /// // Zero-Pad if parts of the memory read fail instead of failing.
    /// // Example assumes: use pretty_hex::*;
    /// let r = vmmprocess.mem_read_ex(va_kernel32, 0x100, FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL);
    /// let Ok(data_read) = r {
    ///     println!("{:?}", data_read.hex_dump());
    /// }
    /// ```
    pub fn mem_read_ex(&self, va : u64, size : usize, flags : u64) -> ResultEx<Vec<u8>> {
        return self.vmm.impl_mem_read(self.pid, va, size, flags);
    }

    /// Read a contigious virtual memory chunk with flags as a type/struct.
    /// 
    /// Flags are constants named `FLAG_*`
    /// 
    /// Reading many memory chunks individually may be slow, especially if
    /// reading takes place using hardware FPGA devices. In that case it's
    /// better to use the `mem_scatter()` functionality for better performance.
    /// 
    /// 
    /// # Arguments
    /// * `va` - Virtual address to start reading from.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// // Read the C-struct IMAGE_DOS_HEADER from memory.
    /// // Force reading the underlying memory device (skip data cache).
    /// #[repr(C)]
    /// struct IMAGE_DOS_HEADER {
    ///     e_magic : u16,
    /// 	...
    ///     e_lfanew : u32,
    /// }
    /// if let Ok(doshdr) = vmmprocess.mem_read_as::<IMAGE_DOS_HEADER>(va_kernel32, FLAG_NOCACHE) {
    ///     println!("e_magic:  {:x}", doshdr.e_magic);
    ///     println!("e_lfanew: {:x}", doshdr.e_lfanew);
    /// }
    /// ```
    pub fn mem_read_as<T>(&self, va : u64, flags : u64) -> ResultEx<T> {
        return self.vmm.impl_mem_read_as(self.pid, va, flags);
    }

    /// Create a scatter memory object for efficient virtual memory reads.
    /// 
    /// Check out the [`VmmScatterMemory`] struct for more detailed information.
    /// 
    /// # Arguments
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// let mem_scatter = vmmprocess.mem_scatter(FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL)?;
    /// ```
    pub fn mem_scatter(&self, flags : u64) -> ResultEx<VmmScatterMemory> {
        return self.vmm.impl_mem_scatter(self.pid, flags);
    }

    /// Translate a virtual address to a physical address.
    /// 
    /// It's not always possible to translate a virtual address to a physical
    /// address. This is the case when memory is "paged out".
    /// 
    /// # Arguments
    /// * `va` - Virtual address to translate.
    /// 
    /// # Examples
    /// ```
    /// let pa_kernel32 = vmmprocess.mem_virt2phys(va_kernel32)?;
    /// ```
    pub fn mem_virt2phys(&self, va : u64) -> ResultEx<u64> {
        return self.vmm.impl_mem_virt2phys(self.pid, va);
    }

    /// Write virtual memory.
    /// 
    /// The write is a best effort. Even of the write should fail it's not
    /// certain that an error will be returned. To be absolutely certain that
    /// a write has taken place follow up with a read.
    /// 
    /// # Arguments
    /// * `va` - Virtual address to start writing from.
    /// * `data` - Byte data to write.
    /// 
    /// # Examples
    /// ```
    /// // Write data starting at the base of kernel32 (in the pe header).
    /// let data_to_write = [0x56u8, 0x4d, 0x4d, 0x52, 0x55, 0x53, 0x54].to_vec();
    /// let _r = vmmprocess.mem_write(va_kernel32, &data_to_write);
    /// ```
    pub fn mem_write(&self, va : u64, data : &Vec<u8>) -> ResultEx<()> {
        return self.vmm.impl_mem_write(self.pid, va, data);
    }

    /// Write a type/struct to virtual memory.
    /// 
    /// The write is a best effort. Even of the write should fail it's not
    /// certain that an error will be returned. To be absolutely certain that
    /// a write has taken place follow up with a read.
    /// 
    /// # Arguments
    /// * `va` - Virtual address to start writing from.
    /// * `data` - Data to write. In case of a struct repr(C) is recommended.
    /// 
    /// # Examples
    /// ```
    /// // Write data starting at the base of kernel32 (in the pe header).
    /// let data_to_write = [0x56, 0x4d, 0x4d, 0x52, 0x55, 0x53, 0x54];
    /// let _r = vmmprocess.mem_write_as(va_kernel32, &data_to_write);
    /// ```
    pub fn mem_write_as<T>(&self, va : u64, data : &T) -> ResultEx<()> {
        return self.vmm.impl_mem_write_as(self.pid, va, data);
    }

    /// Retrieve PDB debugging for the module.
    /// 
    /// PDB debugging most often only work on modules by Microsoft.
    /// See [`VmmPdb`] documentation for additional information.
    /// 
    /// # Arguments
    /// * `va_module_base`
    /// 
    /// # Examples
    /// ```
    /// if let Ok(pdb_kernel32) = vmmprocess.pdb_from_module_address(kernel32.va_base) {
    ///     println!("-> {pdb_kernel32}");
    /// }
    /// ```
    pub fn pdb_from_module_address(&self, va_module_base : u64) -> ResultEx<VmmPdb> {
        return self.impl_pdb_from_module_address(va_module_base);
    }

    /// Retrieve a search struct for process virtual memory.
    /// 
    /// NB! This does not start the actual search yet. 
    /// 
    /// Check out the [`VmmRegValue`] struct for more detailed information.
    /// 
    /// 
    /// # Arguments
    /// * `addr_min` - Start search at this virtual address.
    /// * `addr_max` - End the search at this virtual address. 0 is interpreted as u64::MAX.
    /// * `num_results_max` - Max number of search hits to search for. Max allowed value is 0x10000.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// 
    /// # Examples
    /// ```
    /// // Retrieve a VmmSearch for the entire virtual memory.
    /// let search = vmm.search(0, 0, 0x10000, 0)?
    /// ```
    /// 
    /// ```
    /// // Retrieve a VmmSearch for virtual memory. Stop at first hit.
    /// // Also avoid using cached and paged out memory.
    /// let search = vmm.search(0, 0, 1, FLAG_NOCACHE | FLAG_NOPAGING)?
    /// ```
    pub fn search(&self, addr_min : u64, addr_max : u64, num_results_max : u32, flags : u64) -> ResultEx<VmmSearch> {
        return VmmSearch::impl_new(self.vmm, self.pid, addr_min, addr_max, num_results_max, flags);
    }
}






/// Registry Hive API.
/// 
/// The [`VmmRegHive`] info struct allows for access to the registry hive by
/// exposed fields and various methods.
/// 
/// # Created By
/// - `vmm.reg_hive_list()`
/// 
/// # Examples
/// ```
/// let hive_all = vmm.reg_hive_list()?;
/// for hive in hive_all {
///     println!("{hive} size={} path={}", hive.size, hive.path);
/// }
/// ```
#[derive(Debug)]
pub struct VmmRegHive<'a> {
    vmm : &'a Vmm<'a>,
    pub va : u64,
    pub va_baseblock : u64,
    pub size : u32,
    pub name : String,
    pub name_short : String,
    pub path : String,
}

impl VmmRegHive<'_> {
    /// Read registry hive data.
    /// 
    /// # Arguments
    /// * `ra` - Registry hive address to start reading from.
    /// * `size` - The number of bytes to read.
    /// * `flags` - Any combination of `FLAG_*`.
    /// 
    /// # Examples
    /// ```
    /// if let Ok(data) = hive.reg_hive_read(0x1000, 0x100, FLAG_NOCACHE | FLAG_ZEROPAD_ON_FAIL) {
    ///     println!("{:?}", data.hex_dump());
    /// }
    /// ```
    pub fn reg_hive_read(&self, ra : u32, size : usize, flags : u64) -> ResultEx<Vec<u8>> {
        return self.impl_reg_hive_read(ra, size, flags);
    }

    /// Write registry hive data.
    /// 
    /// Writing to registry hives is extemely unsafe and may lead to
    /// registry corruption and unusable systems. Use with extreme care!
    /// 
    /// # Arguments
    /// * `ra` - Registry hive address to start writing from.
    /// * `data` - Byte data to write.
    /// 
    /// # Examples
    /// ```
    /// let data_to_write = [0x56u8, 0x4d, 0x4d, 0x52, 0x55, 0x53, 0x54].to_vec();
    /// let _r = hive.reg_hive_write(0x1000, &data_to_write);
    /// ```
    pub fn reg_hive_write(&self, ra : u32, data : &Vec<u8>) -> ResultEx<()> {
        return self.impl_reg_hive_write(ra, data);
    }
}

/// Registry Key API.
/// 
/// The [`VmmRegKey`] info struct represents a registry key and also have
/// additional access methods for retrieving registry keys and values.
/// 
/// Registry keys may be addressed either by its full path or by hive address
/// and hive path. Both addressing modes are shown in the examples below.
/// Registry keys are case sensitive.
/// 
/// # Created By
/// - `vmm.reg_key()`
/// - `vmmregkey.parent()`
/// - `vmmregkey.subkeys()`
/// - `vmmregkey.subkeys_map()`
/// - `vmmregvalue.parent()`
/// 
/// # Examples
/// ```
/// // Retrieve a regkey by full path.
/// let regkey = vmm.reg_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run")?
/// println!("{regkey");
/// ```
/// 
/// ```
/// // Retrieve a regkey by hive path.
/// // (SOFTWARE hive example address: 0xffffba061a908000).
/// let regkey = vmm.reg_key("0xffffba061a908000\\ROOT\\Microsoft\\Windows\\CurrentVersion\\Run")?
/// println!("{regkey");
/// ```
#[derive(Debug)]
pub struct VmmRegKey<'a> {
    vmm : &'a Vmm<'a>,
    /// Key name.
    pub name : String,
    /// Path including key name.
    pub path : String,
    /// Last write timestamp in Windows filetime format.
    pub ft_last_write : u64,
}

impl VmmRegKey<'_> {
    /// Retrieve the parent registry key of this registry key.
    /// 
    /// # Examples
    /// ```
    /// let regkey_parent = regkey.parent()?
    /// println!("{regkey_parent");
    /// ```
    pub fn parent(&self) -> ResultEx<VmmRegKey> {
        return self.impl_parent();
    }

    /// Retrieve the registry subkeys of this registry key
    /// 
    /// # Examples
    /// ```
    /// // Retrieve all registry subkeys (as Vec).
    /// let subkeys = regkey.subkeys()?
    /// for key in subkeys {
    ///     println!("{key}")
    /// }
    /// ```
    pub fn subkeys(&self) -> ResultEx<Vec<VmmRegKey>> {
        return self.impl_subkeys();
    }

    /// Retrieve the registry subkeys of this registry key as a map
    /// 
    /// K: String key name,
    /// V: VmmRegKey
    /// 
    /// # Examples
    /// ```
    /// // Retrieve all registry subkeys (as HashMap).
    /// let subkeys = regkey.subkeys_map()?
    /// for e in subkeys {
    ///     println!("{},{}", e.0, e.1)
    /// }
    /// ```
    pub fn subkeys_map(&self) -> ResultEx<HashMap<String, VmmRegKey>> {
        return Ok(self.impl_subkeys()?.into_iter().map(|s| (s.name.clone(), s)).collect());
    }

    /// Retrieve the registry values of this registry key
    /// 
    /// # Examples
    /// ```
    /// // Retrieve all registry values (as Vec).
    /// let values = regkey.values()?
    /// for value in values {
    ///     println!("{value}")
    /// }
    /// ```
    pub fn values(&self) -> ResultEx<Vec<VmmRegValue>> {
        return self.impl_values();
    }

    /// Retrieve the registry values of this registry key as a map
    /// 
    /// K: String value name,
    /// V: VmmRegValue
    /// 
    /// # Examples
    /// ```
    /// // Retrieve all registry values (as HashMap).
    /// let values = regkey.values_map()?
    /// for e in values {
    ///     println!("{},{}", e.0, e.1)
    /// }
    /// ```
    pub fn values_map(&self) -> ResultEx<HashMap<String, VmmRegValue>> {
        return Ok(self.impl_values()?.into_iter().map(|s| (s.name.clone(), s)).collect());
    }

}

#[allow(non_camel_case_types)]
pub enum VmmRegValueType {
    REG_NONE,
    REG_SZ(String),
    REG_EXPAND_SZ(String),
    REG_BINARY(Vec<u8>),
    REG_DWORD(u32),
    REG_DWORD_BIG_ENDIAN(u32),
    REG_LINK(String),
    REG_MULTI_SZ(Vec<String>),
    REG_RESOURCE_LIST(Vec<u8>),
    REG_FULL_RESOURCE_DESCRIPTOR(Vec<u8>),
    REG_RESOURCE_REQUIREMENTS_LIST(Vec<u8>),
    REG_QWORD(u64),
}

/// Registry Value API.
/// 
/// The [`VmmRegValue`] info struct represents a registry value and also have
/// additional access methods for parent key and the value itself.
/// 
/// Registry values may be addressed either by its full path or by hive address
/// and hive path. Both addressing modes are shown in the examples below.
/// Registry values are case sensitive.
/// 
/// # Created By
/// - `vmm.reg_value()`
/// - `vmmregkey.values()`
/// - `vmmregkey.values_map()`
/// 
/// # Examples
/// ```
/// // Retrieve a REG_SZ (string) reg value by its full path.
/// let regpath = "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\ProgramFilesDir";
/// let regvalue = vmm.reg_key(regpath)?
/// println!("{regvalue}");
/// if let Ok(VmmRegValueType::REG_SZ(s)) = regvalue.value() {
///     println!("REG_SZ: {s}");
/// }
/// ```
/// 
/// ```
/// // Retrieve a REG_DWORD reg value using the hive path.
/// // (SOFTWARE hive example address: 0xffffba061a908000).
/// let regpath = "0xffffba061a908000\\ROOT\\Microsoft\\.NETFramework\\Enable64Bit";
/// let regvalue = vmm.reg_key(regpath)?
/// if let Ok(VmmRegValueType::REG_DWORD(dw)) = regvalue.value() {
///     println!("REG_DWORD: 0x{:08x}", dw);
/// }
/// ```
#[derive(Debug)]
pub struct VmmRegValue<'a> {
    vmm : &'a Vmm<'a>,
    /// Value name.
    pub name : String,
    /// Path including key name.
    pub path : String,
    /// The raw type as specified by Windows REG_* constants.
    pub raw_type : u32,
    /// The raw data size in bytes.
    pub raw_size : u32,
    raw_value : Option<Vec<u8>>,
}

impl VmmRegValue<'_> {
    /// Retrieve the parent registry key.
    /// 
    /// # Examples
    /// ```
    /// let regkey_parent = regvalue.parent()?
    /// println!("{regkey_parent");
    /// ```
    pub fn parent(&self) -> ResultEx<VmmRegKey> {
        return self.impl_parent();
    }

    /// Retrieve the registry value.
    /// 
    /// The registry value is returned as [`VmmRegValueType`] enum containing
    /// the relevant embedded value.
    /// 
    /// 
    /// # Examples
    /// ```
    /// // Retrieve a REG_SZ (string) reg value.
    /// if let Ok(VmmRegValueType::REG_SZ(s)) = regvalue.value() {
    ///     println!("REG_SZ: {s}");
    /// }
    /// ```
    /// 
    /// ```
    /// // Retrieve a REG_DWORD reg value.
    /// if let Ok(VmmRegValueType::REG_DWORD(dw)) = regvalue.value() {
    ///     println!("REG_DWORD: 0x{:08x}", dw);
    /// }
    /// ```
    pub fn value(&self) -> ResultEx<VmmRegValueType> {
        return self.impl_value();
    }

    /// Retrieve the raw value bytes backing the actual value.
    /// 
    /// # Examples
    /// ```
    /// let raw_value = vmmregvalue.raw_value()?;
    /// println!("{:?}", raw_value.hex_dump());
    /// ```
    pub fn raw_value(&self) -> ResultEx<Vec<u8>> {
        return self.impl_raw_value();
    }
}






/// Search API.
/// 
/// Search for binary keywords in physical or virtual memory.
/// 
/// Each keyword/term may be up to 32 bytes long. Up to 16 search terms may
/// be used in the same search.
/// 
/// The search may optionally take place with a skipmask - i.e. a bitmask in
/// which '1' would equal a wildcard bit.
/// 
/// The [`VmmSearch`] must be used as mut. Also see [`VmmSearchResult`].
/// 
/// The synchronous search workflow:
/// 1) Acquire search object from `vmm.search()` or `vmmprocess.search()`.
/// 2) Add 1-16 different search terms using `vmmsearch.add_search()` and/or
///    `vmmsearch.add_search_ex()`.
/// 3) Start the search and retrieve result (blocking) by calling
///    `vmmsearch.result()`.
/// 
/// The asynchronous search workflow:
/// 1) Acquire search object from `vmm.search()` or `vmmprocess.search()`.
/// 2) Add 1-16 different search terms using `vmmsearch.add_search()` and/or
///    `vmmsearch.add_search_ex()`.
/// 3) Start the search in the background using `vmmsearch.start()`.
/// 4) Optionally abort the search with `vmmsearch.abort()`.
/// 5) Optionally poll status or result (if completed) using `vmmsearch.poll()`.
/// 6) Optionally retrieve result (blocking) by calling `vmmsearch.result()`.
/// 7) Search goes out of scope and is cleaned up. Any on-going searches may
///    take a short while to terminate gracefully.
/// 
/// 
/// # Created By
/// - `vmm.search()`
/// - `vmmprocess.search()`
/// 
/// # Examples
/// ```
/// // Fetch search struct for entire process virtual address space.
/// // Max 256 search hits and avoid using the cache in this example.
/// let mut vmmsearch = vmmprocess.search(0, 0, 256, FLAG_NOCACHE);
/// // Search for 'MZ' - i.e. start at PE file at even 0x1000 alignment.
/// let search_term = ['M' as u8, 'Z' as u8];
/// let _search_term_id = vmmsearch.add_search_ex(&search_term, None, 0x1000);
/// // Start search in async mode.
/// vmmsearch.start();
/// // Search is now running - it's possible to do other actions here.
/// // It's possible to poll() to see current progress (or if finished).
/// // It's possible to abort() to stop search.
/// // It's possible to fetch result() which will block until search is finished.
/// let search_result = vmmsearch.result();
/// ```
#[derive(Debug)]
pub struct VmmSearch<'a> {
    vmm : &'a Vmm<'a>,
    pid : u32,
    is_started : bool,
    is_completed : bool,
    is_completed_success : bool,
    native_search : CVMMDLL_MEM_SEARCH_CONTEXT,
    thread : Option<std::thread::JoinHandle<bool>>,
    result : Vec<(u64, u32)>,
}

/// Info: Search Progress/Result.
/// 
/// Also see [`VmmSearch`].
/// 
/// # Created By
/// - `vmmsearch.poll()`
/// - `vmmsearch.result()`
/// 
/// # Examples
/// ```
/// // Retrieve a search progress/result in a non-blocking call.
/// let searchresult = vmmsearch.poll();
/// ```
/// 
/// ```
/// // Retrieve a search result in a blocking call (until completed search).
/// let searchresult = vmmsearch.result();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmSearchResult {
    // Indicates that the search has been started. i.e. start() or result() have been called.
    pub is_started : bool,
    // Indicates that the search has been completed.
    pub is_completed : bool,
    // If is_completed is true this indicates if the search was completed successfully.
    pub is_completed_success : bool,
    // Address to start searching from - default 0.
    pub addr_min : u64,
    // Address to stop searching at - default u64::MAX.
    pub addr_max : u64,
    // Current address being searched in search thread.
    pub addr_current : u64,
    // Number of bytes that have been procssed in search.
    pub total_read_bytes : u64,
    // Number of search results.
    pub total_results : u32,
    // The actual result. result.0 = address, result.1 = search_term_id.
    pub result : Vec<(u64, u32)>,
}

impl VmmSearch<'_> {

    /// Add a search term.
    /// 
    /// The search will later be performed using the whole search term and
    /// without alignment requirements (align = 1 byte).
    /// 
    /// On success the `search_term_id` will be returned. This is the 2nd
    /// field (`searchresulttuple.1`) in the search result tuple. This may be
    /// useful if multiple searches are undertaken in one single search run.
    /// 
    /// # Arguments
    /// * `search_bytes` - Byte data to search for. Max 32 bytes.
    /// 
    /// # Examples
    /// ```
    /// // add a search term for pointer references to address 0x7ffcec973308.
    /// let search_term = [0x08, 0x33, 0x97, 0xec, 0xfc, 0x7f, 0x00, 0x00];
    /// let search_term_id = vmmsearch.add_search(&search_term)?;
    /// ```
    pub fn add_search(&mut self, search_bytes : &[u8]) -> ResultEx<u32> {
        return self.impl_add_search(search_bytes, None, 1);
    }

    /// Add a search term.
    /// 
    /// The search will later be performed using the search term with the
    /// given alignment (typically 1, 2, 4, 8, 16, .. 0x1000) and an optional
    /// skip bitmask in which bit '1' represents a search wildcard value.
    /// 
    /// On success the `search_term_id` will be returned. This is the 2nd
    /// field (`searchresulttuple.1`) in the search result tuple. This may be
    /// useful if multiple searches are undertaken in one single search run.
    /// 
    /// # Arguments
    /// * `search_bytes` - Byte data to search for. Max 32 bytes.
    /// * `search_skipmask` - Optional skipmask (see above). Max search_bytes.len().
    /// * `byte_align` - Byte alignment (see above).
    /// 
    /// # Examples
    /// ```
    /// // Add a search term for pointer references to address 0x7ffcec973308.
    /// // Pointers are 64-bit/8-byte aligned hence the 8-byte alignment.
    /// let search_term = [0x08, 0x33, 0x97, 0xec, 0xfc, 0x7f, 0x00, 0x00];
    /// let search_term_id = vmmsearch.add_search_ex(&search_term, None, 8)?;
    /// ```
    pub fn add_search_ex(&mut self, search_bytes : &[u8], search_skipmask : Option<&[u8]>, byte_align : u32) -> ResultEx<u32> {
        return self.impl_add_search(search_bytes, search_skipmask, byte_align);
    }

    /// Start a search in asynchronous background thread.
    /// 
    /// This is useful since the search may take some time and other work may
    /// be done while waiting for the result.
    /// 
    /// The search will start immediately and the progress (and result, if
    /// finished) may be polled by calling [`poll()`](VmmSearch::poll()).
    /// 
    /// The result may be retrieved by a call to `poll()` or by a blocking
    /// call to [`result()`](VmmSearch::result()) which will return when the
    /// search is completed.
    /// 
    /// # Examples
    /// ```
    /// vmmsearch.start();
    /// ```
    pub fn start(&mut self) {
        self.impl_start();
    }

    /// Abort an on-going search.
    /// 
    /// # Examples
    /// ```
    /// vmmsearch.abort();
    /// ```
    pub fn abort(&mut self) {
        self.impl_abort();
    }

    /// Poll an on-going search for the status/result.
    /// 
    /// Also see [`VmmSearch`] and [`VmmSearchResult`].
    /// 
    /// # Examples
    /// ```
    /// search_status_and_result = vmmsearch.poll();
    /// ```
    pub fn poll(&mut self) -> VmmSearchResult {
        return self.impl_poll();
    }

    /// Retrieve the search result.
    /// 
    /// If the search haven't yet been started it will be started.
    /// The function is blocking and will wait for the search to complete
    /// before the search results are returned.
    /// 
    /// Also see [`VmmSearch`] and [`VmmSearchResult`].
    /// 
    /// # Examples
    /// ```
    /// search_status_and_result = vmmsearch.poll();
    /// ```
    pub fn result(&mut self) -> VmmSearchResult {
        return self.impl_result();
    }
}






/// Initialize plugin information and initialization context.
/// 
/// This should usually be the first call in a `InitializeVmmPlugin()` export.
///
/// See the plugin example for additional documentation.
pub fn new_plugin_initialization<T>(native_h : usize, native_reginfo : usize) -> ResultEx<(VmmPluginInitializationInfo, VmmPluginInitializationContext<T>)> {
    return impl_new_plugin_initialization::<T>(native_h, native_reginfo);
}



/// Plugin Context: Supplied by MemProcFS to plugin callback functions.
/// 
/// Contains the `vmm` field which gives access to the general API.
/// 
/// Contains the `ctxlock` field which gives access to the user-defined generic
/// struct set at plugin initialization.
/// 
/// The `ctxlock` field is a `std::sync::RwLock` and the inner user-defined
/// generic struct may be accessed in either multi-threaded read-mode or
/// single-threaded mutable write-mode. Read mode is more efficient.
/// 
/// See the plugin example for additional use cases and documentation.
/// 
/// 
/// # Created By
/// - `plugin sub-system`
/// 
/// 
/// # Examples
/// 
/// ```
/// // Access the `vmm` field to retrieve a process for pid 768.
/// // Some `vmm` calls such as `vmm.process(pid)` may fail. In this case if
/// // the process does not exist. It is recommended to handle these errors
/// // gracefully as per below.
/// if let Ok(systemprocess) = plugin_ctx.vmm.process(768) {
///     // ...
/// }
/// ```
/// 
/// ```
/// // Access the `vmm` field to retrieve a process for pid 768.
/// // Some `vmm` calls such as `vmm.process(pid)` may fail. It is possible to
/// // use error propagation for simplicity. Errors will be handled by upper
/// // plugin layers. If this is preferred error propagation may be simpler.
/// let systemprocess = plugin_ctx.vmm.process(768)?;
/// ```
/// 
/// ```
/// // Access the ctxlock in multi-threaded read-mode:
/// // The lock should always contain a generic so unwrap() should be safe.
/// let user_ctx = plugin_ctx.ctxlock.read().unwrap();
/// ```
/// 
/// ```
/// // Access the ctxlock in single-threaded mutable write-mode:
/// // The lock should always contain a generic so unwrap() should be safe.
/// let mut user_ctx = plugin_ctx.ctxlock.write().unwrap();
/// ```
/// 
/// 
/// See the plugin example about usage of the ctxlock field.
pub struct VmmPluginContext<'a, T> {
    /// Access the general MemProcFS API through the `vmm` field.
    pub vmm     : Vmm<'a>,
    /// Access generic user-set plugin context in a thread-safe way.
    pub ctxlock : std::sync::RwLock<T>,
    fn_list     : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>, path : &str, file_list : &VmmPluginFileList) -> ResultEx<()>>,
    fn_read     : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>, file_name : &str, cb : u32, cb_offset : u64) -> ResultEx<Vec<u8>>>,
    fn_write    : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>, file_name : &str, data : Vec<u8>, cb_offset : u64) -> ResultEx<()>>,
    fn_visible  : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>) -> ResultEx<bool>>,
    fn_notify   : Option<fn(ctxp : &VmmPluginContext<T>, event_id : u32) -> ResultEx<()>>,
}



/// Plugin File List: Supplied by MemProcFS to plugin list callback function.
/// 
/// The `VmmPluginFileList` struct contains the methods `add_file()` and 
/// `add_directory()` which will allow the plugin list callback function
/// to populate files & directories given the specified path and process.
/// 
/// # Created By
/// - `plugin sub-system`
#[derive(Debug)]
pub struct VmmPluginFileList<'a> {
    vmm : &'a Vmm<'a>,
    h_file_list : usize,
}

impl VmmPluginFileList<'_> {
    /// Add a file to the plugin directory indicated by path and process.
    /// 
    /// For additional information check the `plugin_list_cb()` function in the
    /// plugin example project.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Add a directory named readme.txt with size 4kB to the plugin path.
    /// file_list.impl_add_file("readme.txt", 4096);
    /// ```
    pub fn add_file(&self, name : &str, size : u64) {
        self.impl_add_file(name, size);
    }

    /// Add a directory to the plugin directory indicated by path and process.
    /// 
    /// For additional information check the `plugin_list_cb()` function in the
    /// plugin example project.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Add a directory named subdir33 to the plugin path.
    /// file_list.add_directory("subdir33");
    /// ```
    pub fn add_directory(&self, name : &str) {
        self.impl_add_directory(name);
    }
}



/// Plugin Initialization System Information.
/// 
/// The `VmmPluginInitializationInfo` is used in the plugin module entry point
/// (the exported `InitializeVmmPlugin()` function).
/// 
/// The `InitializeVmmPlugin()` function must be fast for the user experience
/// and the initialization function may query this info struct to decide if
/// the current system is supported or not before registering the plugin. 
/// 
/// Contains information about the: system type, memory model and OS version
/// (in the form of build, major and minor).
/// 
/// For additional information check the `InitializeVmmPlugin()` function in
/// the plugin example project.
/// 
/// 
/// # Created By
/// - [`new_plugin_initialization()`]
/// 
/// 
/// # Examples
/// 
/// ```
/// // Retrieve the system_info and plugin_init_ctx in InitializeVmmPlugin()
/// let (system_info, mut plugin_init_ctx) = match new_plugin_initialization::<PluginContext>(native_h, native_reginfo) {
///     Ok(r) => r,
///     Err(_) => return,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmPluginInitializationInfo {
    /// The system type - i.e. 32-bit or 64-bit Windows.
    pub tp_system : VmmSystemType,
    /// The memory model type - i.e. X86, X86PAE, X64.
    pub tp_memorymodel : VmmMemoryModelType,
    /// The OS major version. Use version_build instead if possible.
    pub version_major : u32,
    /// The OS minor version. Use version_build instead if possible.
    pub version_minor : u32,
    /// The build version number.
    pub version_build : u32,
}



/// Plugin Initialization Context.
/// 
/// The `VmmPluginInitializationContext` is used in the plugin module entry
/// point (the exported `InitializeVmmPlugin()` function).
/// 
/// The context is to be populated by the user with information such as name,
/// callback functions and plugin visibility.
/// 
/// The flow usually follows the below structure:
/// 
/// 1: Call: memprocfs::new_plugin_initialization(native_h, native_reginfo) to
///    create the plugin init context inside the InitializeVmmPlugin() function.
/// 
/// 2: Fill out the required ctx and path_name struct members.
/// 
/// 3: Fill out the module type in the is* struct members.
/// 
/// 4: Fill out the optional pfn* callback functions.
/// 
/// 5: Register the plugin with the VMM by calling the register() method.
/// 
/// For additional information check the `InitializeVmmPlugin()` function in
/// the plugin example project.
/// 
/// 
/// # Created By
/// - [`new_plugin_initialization()`]
/// 
/// 
/// # Examples
/// 
/// ```
/// // Retrieve the system_info and plugin_init_ctx in InitializeVmmPlugin()
/// let (system_info, mut plugin_init_ctx) = match new_plugin_initialization::<PluginContext>(native_h, native_reginfo) {
///     Ok(r) => r,
///     Err(_) => return,
/// };
/// // set plugin name:
/// plugin_init_ctx.path_name = String::from("/rust/example");
/// // Set user-defined generic plugin context:
/// let ctx = PluginContext {
///     ...
/// };
/// plugin_init_ctx.ctx = Some(ctx);
/// // Set visiblity:
/// plugin_init_ctx.is_root_module = true;
/// plugin_init_ctx.is_process_module = true;
/// // Set callback functions:
/// plugin_init_ctx.fn_list = Some(plugin_list_cb);
/// plugin_init_ctx.fn_read = Some(plugin_read_cb);
/// plugin_init_ctx.fn_write = Some(plugin_write_cb);
/// // Register the plugin with the MemProcFS plugin manager:
/// let _r = plugin_init_ctx.register();
/// ```
pub struct VmmPluginInitializationContext<T> {
    h_vmm           : usize,
    h_reginfo       : usize,
    /// user-defined generic plugin context.
    pub ctx         : Option<T>,
    /// Plugin path and name.
    pub path_name   : String,
    /// Plugin shows up in the file system root.
    pub is_root_module : bool,
    /// Plugin is hidden in the file system root.
    pub is_root_module_hidden : bool,
    /// Plugin shows up on a per-process basis.
    pub is_process_module : bool,
    /// Plugin is hidden on a per-process basis.
    pub is_process_module_hidden : bool,
    /// Callback function - VFS list directory. This callback used in most cases.
    pub fn_list    : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>, path : &str, file_list : &VmmPluginFileList) -> ResultEx<()>>,
    /// Callback function - VFS read file. This callback is used in most cases.
    pub fn_read    : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>, file_name : &str, cb : u32, cb_offset : u64) -> ResultEx<Vec<u8>>>,
    /// Callback function - VFS write file.
    pub fn_write   : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>, file_name : &str, data : Vec<u8>, cb_offset : u64) -> ResultEx<()>>,
    /// Callback function - plugin dynamic visiblity. This callback is rarely used, and in special circumstances only.
    pub fn_visible : Option<fn(ctxp : &VmmPluginContext<T>, process : Option<VmmProcess>) -> ResultEx<bool>>,
    /// Callback function - notification on an event defined by: `PLUGIN_NOTIFY_*` constants.
    pub fn_notify  : Option<fn(ctxp : &VmmPluginContext<T>, event_id : u32) -> ResultEx<()>>,
}

impl<T> VmmPluginInitializationContext<T> {
    /// Register the plugin with the MemProcFS plugin sub-system.
    /// 
    /// The initialiation context may not be used after the `register()` call.
    /// 
    /// It is possible to register additional plugins in the same plugin
    /// initialization function if a new `VmmPluginInitializationContext`
    /// is retrieved from the `new_plugin_initialization()` function.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Register the plugin with MemProcFS. This will consume the context
    /// // which should not be possible to use after this.
    /// let _r = plugin_init_ctx.register();
    /// ```
    /// 
    pub fn register(self) -> ResultEx<()> {
        return self.impl_register();
    }
}





















//=============================================================================
// INTERNAL: VMM CORE:
//=============================================================================

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Debug)]
struct VmmNative {
    h : usize,
    is_close_h : bool,
    library_lc : Option<libloading::Library>,
    library_vmm : Option<libloading::Library>,
    VMMDLL_Initialize :             extern "C" fn(argc: c_int, argv: *const *const c_char) -> usize,
    VMMDLL_InitializePlugins :      extern "C" fn(hVMM : usize) -> bool,
    VMMDLL_Close :                  extern "C" fn(hVMM : usize),
    VMMDLL_ConfigGet :              extern "C" fn(hVMM : usize, fOption : u64, pqwValue : *mut u64) -> bool,
    VMMDLL_ConfigSet :              extern "C" fn(hVMM : usize, fOption : u64, qwValue : u64) -> bool,
    VMMDLL_MemFree :                extern "C" fn(pvMem : usize),
    
    VMMDLL_Log :                    extern "C" fn(hVMM : usize, MID : u32, dwLogLevel : u32, uszFormat : *const c_char, uszParam : *const c_char),
    VMMDLL_MemSearch :              extern "C" fn(hVMM : usize, pid : u32, ctx : *mut CVMMDLL_MEM_SEARCH_CONTEXT, ppva : *mut u64, pcva : *mut u32) -> bool,

    VMMDLL_MemReadEx :              extern "C" fn(hVMM : usize, pid : u32, qwA : u64, pb : *mut u8, cb : u32, pcbReadOpt : *mut u32, flags : u64) -> bool,
    VMMDLL_MemWrite :               extern "C" fn(hVMM : usize, pid : u32, qwA : u64, pb : *const u8, cb : u32) -> bool,
    VMMDLL_MemVirt2Phys :           extern "C" fn(hVMM : usize, pid : u32, qwA : u64, pqwPA : *mut u64) -> bool,

    VMMDLL_Scatter_Initialize :     extern "C" fn(hVMM : usize, pid : u32, flags : u32) -> usize,
    VMMDLL_Scatter_Prepare :        extern "C" fn(hS : usize, va : u64, cb : u32) -> bool,
    VMMDLL_Scatter_PrepareEx :      extern "C" fn(hS : usize, va : u64, cb : u32, pb : *mut u8, pcbRead : *mut u32) -> bool,
    VMMDLL_Scatter_PrepareWrite :   extern "C" fn(hS : usize, va : u64, pb : *const u8, cb : u32) -> bool,
    VMMDLL_Scatter_Execute :        extern "C" fn(hS : usize) -> bool,
    VMMDLL_Scatter_Read :           extern "C" fn(hS : usize, va : u64, cb : u32, pb : *mut u8, pcbRead : *mut u32) -> bool,
    VMMDLL_Scatter_Clear :          extern "C" fn(hS : usize, pid : u32, flags : u32) -> bool,
    VMMDLL_Scatter_CloseHandle :    extern "C" fn(hS : usize),

    VMMDLL_PidGetFromName :         extern "C" fn(hVMM : usize, szProcName : *const c_char, pdwPID : *mut u32) -> bool,
    VMMDLL_PidList :                extern "C" fn(hVMM : usize, pPIDs : *mut u32, pcPIDs : *mut usize) -> bool,

    VMMDLL_WinReg_HiveList :        extern "C" fn(hVMM : usize, pHives : *mut CRegHive, cHives : u32, pcHives : *mut u32) -> bool,
    VMMDLL_WinReg_HiveReadEx :      extern "C" fn(hVMM : usize, vaCMHive : u64, ra : u32, pb : *mut u8, cb : u32, pcbReadOpt : *mut u32, flags : u64) -> bool,
    VMMDLL_WinReg_HiveWrite :       extern "C" fn(hVMM : usize, vaCMHive : u64, ra : u32, pb : *const u8, cb : u32) -> bool,
    VMMDLL_WinReg_EnumKeyExU :      extern "C" fn(hVMM : usize, uszFullPathKey : *const c_char, dwIndex : u32, lpcchName : *mut c_char, lpcchName : *mut u32, lpftLastWriteTime : *mut u64) -> bool,
    VMMDLL_WinReg_EnumValueU :      extern "C" fn(hVMM : usize, uszFullPathKey : *const c_char, dwIndex : u32, lpValueName : *mut c_char, lpcchValueName : *mut u32, lpType : *mut u32, lpcbData : *mut u32) -> bool,
    VMMDLL_WinReg_QueryValueExU :   extern "C" fn(hVMM : usize, uszFullPathKeyValue : *const c_char, lpType : *mut u32, lpData : *mut u8, lpcbData : *mut u32) -> bool,

    VMMDLL_ProcessGetModuleBaseU :  extern "C" fn(hVMM : usize, pid : u32, uszModuleName : *const c_char) -> u64,
    VMMDLL_ProcessGetProcAddressU : extern "C" fn(hVMM : usize, pid : u32, uszModuleName : *const c_char, szFunctionName : *const c_char) -> u64,
    VMMDLL_ProcessGetInformation :  extern "C" fn(hVMM : usize, pid : u32, pProcessInformation : *mut CProcessInformation, pcbProcessInformation : *mut usize) -> bool,
    VMMDLL_ProcessGetInformationString : extern "C" fn(hVMM : usize, pid : u32, fOptionString : u32) -> *const c_char,

    VMMDLL_Map_GetNetU :            extern "C" fn(hVMM : usize, ppNetMap : *mut *mut CNetMap) -> bool,
    VMMDLL_Map_GetPfnEx :           extern "C" fn(hVMM : usize, pPfns : *const u32, cPfns : u32, ppPfnMap : *mut *mut CPfnMap, flags : u32) -> bool,
    VMMDLL_Map_GetPhysMem :         extern "C" fn(hVMM : usize, ppPhysMemMap : *mut *mut CMemoryMap) -> bool,
    VMMDLL_Map_GetPool :            extern "C" fn(hVMM : usize, ppPoolMap : *mut *mut CPoolMap, flags : u32) -> bool,
    VMMDLL_Map_GetServicesU :       extern "C" fn(hVMM : usize, ppServiceMap : *mut *mut CServiceMap) -> bool,
    VMMDLL_Map_GetUsersU :          extern "C" fn(hVMM : usize, ppUserMap : *mut *mut CUserMap) -> bool,
    VMMDLL_Map_GetVMU :             extern "C" fn(hVMM : usize, ppVmMap : *mut *mut CVmMap) -> bool,

    VMMDLL_PdbLoad :                extern "C" fn(hVMM : usize, dwPID : u32, vaModuleBase : u64, szModuleName : *mut c_char) -> bool,
    VMMDLL_PdbSymbolName :          extern "C" fn(hVMM : usize, szModule : *const c_char, cbSymbolAddressOrOffset : u64, szSymbolName : *mut c_char, pdwSymbolDisplacement : *mut u32) -> bool,
    VMMDLL_PdbSymbolAddress :       extern "C" fn(hVMM : usize, szModule : *const c_char, szSymbolName : *const c_char, pvaSymbolAddress : *mut u64) -> bool,
    VMMDLL_PdbTypeSize :            extern "C" fn(hVMM : usize, szModule : *const c_char, szTypeName : *const c_char, pcbTypeSize : *mut u32) -> bool,
    VMMDLL_PdbTypeChildOffset :     extern "C" fn(hVMM : usize, szModule : *const c_char, uszTypeName : *const c_char, uszTypeChildName : *const c_char, pcbTypeChildOffset : *mut u32) -> bool,

    VMMDLL_Map_GetEATU :            extern "C" fn(hVMM : usize, pid : u32, uszModuleName : *const c_char, ppEatMap : *mut *mut CEatMap) -> bool,
    VMMDLL_Map_GetHandleU :         extern "C" fn(hVMM : usize, pid : u32, ppHandleMap : *mut *mut CHandleMap) -> bool,
    VMMDLL_Map_GetHeap :            extern "C" fn(hVMM : usize, pid : u32, ppHeapMap : *mut *mut CHeapMap) -> bool,
    VMMDLL_Map_GetHeapAlloc :       extern "C" fn(hVMM : usize, pid : u32, qwHeapNumOrAddress : u64, ppHeapAllocMap : *mut *mut CHeapAllocMap) -> bool,
    VMMDLL_Map_GetIATU :            extern "C" fn(hVMM : usize, pid : u32, uszModuleName : *const c_char, ppIatMap : *mut *mut CIatMap) -> bool,
    VMMDLL_Map_GetModuleU :         extern "C" fn(hVMM : usize, pid : u32, ppModuleMap : *mut *mut CModuleMap, flags : u32) -> bool,
    VMMDLL_Map_GetPteU :            extern "C" fn(hVMM : usize, pid : u32, fIdentifyModules : bool, ppPteMap : *mut *mut CPteMap) -> bool,
    VMMDLL_Map_GetThread :          extern "C" fn(hVMM : usize, pid : u32, ppThreadMap : *mut *mut CThreadMap) -> bool,
    VMMDLL_Map_GetUnloadedModuleU : extern "C" fn(hVMM : usize, pid : u32, ppUnloadedModuleMap : *mut *mut CUnloadedModuleMap) -> bool,
    VMMDLL_Map_GetVadU :            extern "C" fn(hVMM : usize, pid : u32, fIdentifyModules : bool, ppVadMap : *mut *mut CVadMap) -> bool,
    VMMDLL_Map_GetVadEx :           extern "C" fn(hVMM : usize, pid : u32, oPage : u32, cPage : u32, ppVadExMap : *mut *mut CVadExMap) -> bool,
    VMMDLL_ProcessGetDirectoriesU : extern "C" fn(hVMM : usize, pid : u32, uszModule : *const c_char, pDataDirectories : *mut CIMAGE_DATA_DIRECTORY) -> bool,
    VMMDLL_ProcessGetSectionsU :    extern "C" fn(hVMM : usize, pid : u32, uszModule : *const c_char, pSections : *mut CIMAGE_SECTION_HEADER, cSections : u32, pcSections : *mut u32) -> bool,

    VMMDLL_VfsListU :               extern "C" fn(hVMM : usize, uszPath : *const c_char, pFileList : *mut CVMMDLL_VFS_FILELIST2) -> bool,
    VMMDLL_VfsReadU :               extern "C" fn(hVMM : usize, uszFileName : *const c_char, pb : *mut u8, cb : u32, pcbRead : *mut u32, cbOffset : u64) -> u32,
    VMMDLL_VfsWriteU :              extern "C" fn(hVMM : usize, uszFileName : *const c_char, pb : *const u8, cb : u32, pcbWrite : *mut u32, cbOffset : u64) -> u32,

    VMMDLL_VmGetVmmHandle :         extern "C" fn(hVMM : usize, hVM : usize) -> usize,

    // Plugin related info below:
    VMMDLL_VfsList_AddFile :        extern "C" fn(pFileList : usize, uszName : *const c_char, cb : u64, pExInfo : usize),
    VMMDLL_VfsList_AddDirectory :   extern "C" fn(pFileList : usize, uszName : *const c_char, pExInfo : usize),

}

#[allow(non_snake_case)]
fn impl_new<'a>(vmm_lib_path : &str, h_vmm_existing_opt : usize, args: &Vec<&str>) -> ResultEx<Vmm<'a>> {
    unsafe {
        // load MemProcFS native library (vmm.dll / vmm.so):
        // vmm is however dependant on leechcore which must be loaded first...
        let path_vmm = std::path::Path::new(vmm_lib_path).canonicalize()?;
        let mut path_lc = path_vmm.parent().unwrap().canonicalize()?;
        if cfg!(windows) {
            path_lc = path_lc.join("leechcore.dll");
        } else {
            path_lc = path_lc.join("leechcore.so");
        }
        let lib_lc : libloading::Library = libloading::Library::new(path_lc.to_str().unwrap_or(""))?;
        let lib : libloading::Library = libloading::Library::new(path_vmm.to_str().unwrap_or(""))?;
        // fetch function references:
        let VMMDLL_Initialize : extern "C" fn(argc: c_int, argv: *const *const c_char) -> usize = *lib.get(b"VMMDLL_Initialize")?;
        let VMMDLL_InitializePlugins : extern "C" fn(usize) -> bool = *lib.get(b"VMMDLL_InitializePlugins")?;
        let VMMDLL_Close = *lib.get(b"VMMDLL_Close")?;
        let VMMDLL_ConfigGet = *lib.get(b"VMMDLL_ConfigGet")?;
        let VMMDLL_ConfigSet = *lib.get(b"VMMDLL_ConfigSet")?;
        let VMMDLL_MemFree = *lib.get(b"VMMDLL_MemFree")?;
        let VMMDLL_Log = *lib.get(b"VMMDLL_Log")?;
        let VMMDLL_MemSearch = *lib.get(b"VMMDLL_MemSearch")?;
        let VMMDLL_MemReadEx = *lib.get(b"VMMDLL_MemReadEx")?;
        let VMMDLL_MemWrite = *lib.get(b"VMMDLL_MemWrite")?;
        let VMMDLL_MemVirt2Phys = *lib.get(b"VMMDLL_MemVirt2Phys")?;
        let VMMDLL_Scatter_Initialize = *lib.get(b"VMMDLL_Scatter_Initialize")?;
        let VMMDLL_Scatter_Prepare = *lib.get(b"VMMDLL_Scatter_Prepare")?;
        let VMMDLL_Scatter_PrepareEx = *lib.get(b"VMMDLL_Scatter_PrepareEx")?;
        let VMMDLL_Scatter_PrepareWrite = *lib.get(b"VMMDLL_Scatter_PrepareWrite")?;
        let VMMDLL_Scatter_Execute = *lib.get(b"VMMDLL_Scatter_Execute")?;
        let VMMDLL_Scatter_Read = *lib.get(b"VMMDLL_Scatter_Read")?;
        let VMMDLL_Scatter_Clear = *lib.get(b"VMMDLL_Scatter_Clear")?;
        let VMMDLL_Scatter_CloseHandle = *lib.get(b"VMMDLL_Scatter_CloseHandle")?;
        let VMMDLL_PidGetFromName = *lib.get(b"VMMDLL_PidGetFromName")?;
        let VMMDLL_PidList = *lib.get(b"VMMDLL_PidList")?;
        let VMMDLL_WinReg_HiveList = *lib.get(b"VMMDLL_WinReg_HiveList")?;
        let VMMDLL_WinReg_HiveReadEx = *lib.get(b"VMMDLL_WinReg_HiveReadEx")?;
        let VMMDLL_WinReg_HiveWrite = *lib.get(b"VMMDLL_WinReg_HiveWrite")?;
        let VMMDLL_WinReg_EnumKeyExU = *lib.get(b"VMMDLL_WinReg_EnumKeyExU")?;
        let VMMDLL_WinReg_EnumValueU = *lib.get(b"VMMDLL_WinReg_EnumValueU")?;
        let VMMDLL_WinReg_QueryValueExU = *lib.get(b"VMMDLL_WinReg_QueryValueExU")?;
        let VMMDLL_ProcessGetModuleBaseU = *lib.get(b"VMMDLL_ProcessGetModuleBaseU")?;
        let VMMDLL_ProcessGetProcAddressU = *lib.get(b"VMMDLL_ProcessGetProcAddressU")?;
        let VMMDLL_ProcessGetInformation = *lib.get(b"VMMDLL_ProcessGetInformation")?;
        let VMMDLL_ProcessGetInformationString = *lib.get(b"VMMDLL_ProcessGetInformationString")?;
        let VMMDLL_Map_GetNetU = *lib.get(b"VMMDLL_Map_GetNetU")?;
        let VMMDLL_Map_GetPfnEx = *lib.get(b"VMMDLL_Map_GetPfnEx")?;
        let VMMDLL_Map_GetPhysMem = *lib.get(b"VMMDLL_Map_GetPhysMem")?;
        let VMMDLL_Map_GetPool = *lib.get(b"VMMDLL_Map_GetPool")?;
        let VMMDLL_Map_GetUsersU = *lib.get(b"VMMDLL_Map_GetUsersU")?;
        let VMMDLL_Map_GetServicesU = *lib.get(b"VMMDLL_Map_GetServicesU")?;
        let VMMDLL_Map_GetVMU = *lib.get(b"VMMDLL_Map_GetVMU")?;
        let VMMDLL_PdbLoad = *lib.get(b"VMMDLL_PdbLoad")?;
        let VMMDLL_PdbSymbolName = *lib.get(b"VMMDLL_PdbSymbolName")?;
        let VMMDLL_PdbSymbolAddress = *lib.get(b"VMMDLL_PdbSymbolAddress")?;
        let VMMDLL_PdbTypeSize = *lib.get(b"VMMDLL_PdbTypeSize")?;
        let VMMDLL_PdbTypeChildOffset = *lib.get(b"VMMDLL_PdbTypeChildOffset")?;
        let VMMDLL_Map_GetEATU = *lib.get(b"VMMDLL_Map_GetEATU")?;
        let VMMDLL_Map_GetHandleU = *lib.get(b"VMMDLL_Map_GetHandleU")?;
        let VMMDLL_Map_GetHeap = *lib.get(b"VMMDLL_Map_GetHeap")?;
        let VMMDLL_Map_GetHeapAlloc = *lib.get(b"VMMDLL_Map_GetHeapAlloc")?;
        let VMMDLL_Map_GetIATU = *lib.get(b"VMMDLL_Map_GetIATU")?;
        let VMMDLL_Map_GetModuleU = *lib.get(b"VMMDLL_Map_GetModuleU")?;
        let VMMDLL_Map_GetPteU = *lib.get(b"VMMDLL_Map_GetPteU")?;
        let VMMDLL_Map_GetThread = *lib.get(b"VMMDLL_Map_GetThread")?;
        let VMMDLL_Map_GetUnloadedModuleU = *lib.get(b"VMMDLL_Map_GetUnloadedModuleU")?;
        let VMMDLL_Map_GetVadU = *lib.get(b"VMMDLL_Map_GetVadU")?;
        let VMMDLL_Map_GetVadEx = *lib.get(b"VMMDLL_Map_GetVadEx")?;
        let VMMDLL_ProcessGetDirectoriesU = *lib.get(b"VMMDLL_ProcessGetDirectoriesU")?;
        let VMMDLL_ProcessGetSectionsU = *lib.get(b"VMMDLL_ProcessGetSectionsU")?;
        let VMMDLL_VfsListU = *lib.get(b"VMMDLL_VfsListU")?;
        let VMMDLL_VfsReadU = *lib.get(b"VMMDLL_VfsReadU")?;
        let VMMDLL_VfsWriteU = *lib.get(b"VMMDLL_VfsWriteU")?;
        let VMMDLL_VmGetVmmHandle = *lib.get(b"VMMDLL_VmGetVmmHandle")?;
        let VMMDLL_VfsList_AddFile = *lib.get(b"VMMDLL_VfsList_AddFile")?;
        let VMMDLL_VfsList_AddDirectory = *lib.get(b"VMMDLL_VfsList_AddDirectory")?;
        // initialize MemProcFS
        let h;
        if h_vmm_existing_opt != 0 {
            h = h_vmm_existing_opt;
        } else {
            let args = args.iter().map(|arg| CString::new(*arg).unwrap()).collect::<Vec<CString>>();
            let argv: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
            let argc: c_int = args.len() as c_int;
            h = (VMMDLL_Initialize)(argc, argv.as_ptr());
            if h == 0 {
                return Err("VMMDLL_Initialize: fail".into());
            }
            let r = (VMMDLL_InitializePlugins)(h);
            if !r {
                return Err("VMMDLL_InitializePlugins: fail".into());
            }
        }
        // return Vmm struct:
        let native = VmmNative {
            h,
            is_close_h : h_vmm_existing_opt == 0,
            library_lc : Some(lib_lc),
            library_vmm : Some(lib),
            VMMDLL_Initialize,
            VMMDLL_InitializePlugins,
            VMMDLL_Close,
            VMMDLL_ConfigGet,
            VMMDLL_ConfigSet,
            VMMDLL_MemFree,
            VMMDLL_Log,
            VMMDLL_MemSearch,
            VMMDLL_MemReadEx,
            VMMDLL_MemWrite,
            VMMDLL_MemVirt2Phys,
            VMMDLL_Scatter_Initialize,
            VMMDLL_Scatter_Prepare,
            VMMDLL_Scatter_PrepareEx,
            VMMDLL_Scatter_PrepareWrite,
            VMMDLL_Scatter_Execute,
            VMMDLL_Scatter_Read,
            VMMDLL_Scatter_Clear,
            VMMDLL_Scatter_CloseHandle,
            VMMDLL_PidGetFromName,
            VMMDLL_PidList,
            VMMDLL_WinReg_HiveList,
            VMMDLL_WinReg_HiveReadEx,
            VMMDLL_WinReg_HiveWrite,
            VMMDLL_WinReg_EnumKeyExU,
            VMMDLL_WinReg_EnumValueU,
            VMMDLL_WinReg_QueryValueExU,
            VMMDLL_ProcessGetModuleBaseU,
            VMMDLL_ProcessGetProcAddressU,
            VMMDLL_ProcessGetInformation,
            VMMDLL_ProcessGetInformationString,
            VMMDLL_Map_GetNetU,
            VMMDLL_Map_GetPfnEx,
            VMMDLL_Map_GetPhysMem,
            VMMDLL_Map_GetPool,
            VMMDLL_Map_GetUsersU,
            VMMDLL_Map_GetServicesU,
            VMMDLL_Map_GetVMU,
            VMMDLL_PdbLoad,
            VMMDLL_PdbSymbolName,
            VMMDLL_PdbSymbolAddress,
            VMMDLL_PdbTypeSize,
            VMMDLL_PdbTypeChildOffset,
            VMMDLL_Map_GetEATU,
            VMMDLL_Map_GetHandleU,
            VMMDLL_Map_GetHeap,
            VMMDLL_Map_GetHeapAlloc,
            VMMDLL_Map_GetIATU,
            VMMDLL_Map_GetModuleU,
            VMMDLL_Map_GetPteU,
            VMMDLL_Map_GetThread,
            VMMDLL_Map_GetUnloadedModuleU,
            VMMDLL_Map_GetVadU,
            VMMDLL_Map_GetVadEx,
            VMMDLL_ProcessGetDirectoriesU,
            VMMDLL_ProcessGetSectionsU,
            VMMDLL_VfsListU,
            VMMDLL_VfsReadU,
            VMMDLL_VfsWriteU,
            VMMDLL_VmGetVmmHandle,
            VMMDLL_VfsList_AddFile,
            VMMDLL_VfsList_AddDirectory,
        };
        let vmm = Vmm {
            native,
            parent_vmm : None,
        };
        return Ok(vmm);
    }
}

#[allow(non_snake_case)]
fn impl_new_from_virtual_machine<'a>(vmm_parent : &'a Vmm, vm_entry : &VmmMapVirtualMachineEntry) -> ResultEx<Vmm<'a>> {
    if vmm_parent.native.h != vm_entry.h_vmm {
        return Err("Invalid parent/vm relationship.".into());
    }
    let h_vmm_vm = (vmm_parent.native.VMMDLL_VmGetVmmHandle)(vmm_parent.native.h, vm_entry.h_vm);
    if h_vmm_vm == 0 {
        return Err("VMMDLL_VmGetVmmHandle: fail.".into());
    }
    let native = VmmNative {
        h: vmm_parent.native.h,
        library_lc : None,
        library_vmm : None,
        ..vmm_parent.native
    };
    let vmm = Vmm {
        native : native,
        parent_vmm : Some(vmm_parent),
    };
    return Ok(vmm);
}






//=============================================================================
// INTERNAL: VMM:
//=============================================================================

const MAX_PATH                          : usize = 260;
const VMMDLL_MEM_SEARCH_VERSION         : u32 = 0xfe3e0002;
const VMMDLL_VFS_FILELIST_VERSION       : u32 = 2;
const VMMDLL_MAP_EAT_VERSION            : u32 = 3;
const VMMDLL_MAP_HANDLE_VERSION         : u32 = 3;
const VMMDLL_MAP_HEAP_VERSION           : u32 = 4;
const VMMDLL_MAP_HEAPALLOC_VERSION      : u32 = 1;
const VMMDLL_MAP_IAT_VERSION            : u32 = 2;
const VMMDLL_MAP_POOL_VERSION           : u32 = 2;
const VMMDLL_MAP_PTE_VERSION            : u32 = 2;
const VMMDLL_MAP_MODULE_VERSION         : u32 = 6;
const VMMDLL_MAP_NET_VERSION            : u32 = 3;
const VMMDLL_MAP_PFN_VERSION            : u32 = 1;
const VMMDLL_MAP_PHYSMEM_VERSION        : u32 = 2;
const VMMDLL_MAP_SERVICE_VERSION        : u32 = 3;
const VMMDLL_MAP_THREAD_VERSION         : u32 = 4;
const VMMDLL_MAP_UNLOADEDMODULE_VERSION : u32 = 2;
const VMMDLL_MAP_USER_VERSION           : u32 = 2;
const VMMDLL_MAP_VAD_VERSION            : u32 = 6;
const VMMDLL_MAP_VADEX_VERSION          : u32 = 3;
const VMMDLL_MAP_VM_VERSION             : u32 = 2;

const VMMDLL_MID_RUST                   : u32 = 0x80000004;

const VMMDLL_PLUGIN_CONTEXT_MAGIC       : u64 = 0xc0ffee663df9301c;
const VMMDLL_PLUGIN_CONTEXT_VERSION     : u16 = 5;
const VMMDLL_PLUGIN_REGINFO_MAGIC       : u64 = 0xc0ffee663df9301d;
const VMMDLL_PLUGIN_REGINFO_VERSION     : u16 = 16;
const VMMDLL_STATUS_SUCCESS             : u32 = 0x00000000;
const VMMDLL_STATUS_END_OF_FILE         : u32 = 0xC0000011;
const VMMDLL_STATUS_FILE_INVALID        : u32 = 0xC0000098;

const VMMDLL_PROCESS_INFORMATION_MAGIC          : u64 = 0xc0ffee663df9301e;
const VMMDLL_PROCESS_INFORMATION_VERSION        : u16 = 7;
const VMMDLL_REGISTRY_HIVE_INFORMATION_MAGIC    : u64 = 0xc0ffee653df8d01e;
const VMMDLL_REGISTRY_HIVE_INFORMATION_VERSION  : u16 = 4;

const VMMDLL_PROCESS_INFORMATION_OPT_STRING_PATH_KERNEL     : u32 = 1;
const VMMDLL_PROCESS_INFORMATION_OPT_STRING_PATH_USER_IMAGE : u32 = 2;
const VMMDLL_PROCESS_INFORMATION_OPT_STRING_CMDLINE         : u32 = 3;

const DIRECTORY_NAMES : [&str; 16] = ["EXPORT",  "IMPORT",  "RESOURCE",  "EXCEPTION",  "SECURITY",  "BASERELOC",  "DEBUG",  "ARCHITECTURE",  "GLOBALPTR",  "TLS",  "LOAD_CONFIG",  "BOUND_IMPORT",  "IAT",  "DELAY_IMPORT",  "COM_DESCRIPTOR",  "RESERVED"];

impl Drop for Vmm<'_> {
    fn drop(&mut self) {
        if self.native.is_close_h {
            (self.native.VMMDLL_Close)(self.native.h);
        }
    }
}

impl fmt::Display for Vmm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vmm")
    }
}

impl fmt::Display for VmmLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmLogLevel::_1Critical => "Critical(1)",
            VmmLogLevel::_2Warning => "Warning(2)",
            VmmLogLevel::_3Info => "Info(3)",
            VmmLogLevel::_4Verbose => "Verbose(4)",
            VmmLogLevel::_5Debug => "Debug(5)",
            VmmLogLevel::_6Trace => "Trace(6)",
            VmmLogLevel::_7None => "None(7)",
        };
        write!(f, "{v}")
    }
}

impl From<u32> for VmmMemoryModelType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmMemoryModelType::X86,
            2 => VmmMemoryModelType::X86PAE,
            3 => VmmMemoryModelType::X64,
            _ => VmmMemoryModelType::NA,
        };
    }
}

impl fmt::Display for VmmMemoryModelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmMemoryModelType::NA => "NA",
            VmmMemoryModelType::X86 => "X86",
            VmmMemoryModelType::X86PAE => "X86PAE",
            VmmMemoryModelType::X64 => "X64",
        };
        write!(f, "{v}")
    }
}

impl From<u32> for VmmSystemType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmSystemType::UnknownX64,
            2 => VmmSystemType::WindowsX64,
            3 => VmmSystemType::UnknownX86,
            4 => VmmSystemType::WindowsX86,
            _ => VmmSystemType::UnknownPhysical,
        };
    }
}

impl fmt::Display for VmmSystemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmSystemType::UnknownPhysical => "UnknownPhysical",
            VmmSystemType::UnknownX64 => "UnknownX64",
            VmmSystemType::WindowsX64 => "WindowsX64",
            VmmSystemType::UnknownX86 => "UnknownX86",
            VmmSystemType::WindowsX86 => "WindowsX86",
        };
        write!(f, "{v}")
    }
}

impl From<u32> for VmmIntegrityLevelType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmIntegrityLevelType::Untrusted,
            2 => VmmIntegrityLevelType::Low,
            3 => VmmIntegrityLevelType::Medium,
            4 => VmmIntegrityLevelType::MediumPlus,
            5 => VmmIntegrityLevelType::High,
            6 => VmmIntegrityLevelType::System,
            7 => VmmIntegrityLevelType::Protected,
            _ => VmmIntegrityLevelType::Unknown,
        };
    }
}

impl fmt::Display for VmmIntegrityLevelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmIntegrityLevelType::Untrusted => "Untrusted",
            VmmIntegrityLevelType::Low => "Low",
            VmmIntegrityLevelType::Medium => "Medium",
            VmmIntegrityLevelType::MediumPlus => "MediumPlus",
            VmmIntegrityLevelType::High => "High",
            VmmIntegrityLevelType::System => "System",
            VmmIntegrityLevelType::Protected => "Protected",
            VmmIntegrityLevelType::Unknown => "Unknown",
        };
        write!(f, "{v}")
    }
}

impl From<u32> for VmmMapPfnType {
    fn from(v : u32) -> Self {
        return match v {
            0 => VmmMapPfnType::Zero,
            1 => VmmMapPfnType::Free,
            2 => VmmMapPfnType::Standby,
            3 => VmmMapPfnType::Modified,
            4 => VmmMapPfnType::ModifiedNoWrite,
            5 => VmmMapPfnType::Bad,
            6 => VmmMapPfnType::Active,
            _ => VmmMapPfnType::Transition,
        };
    }
}

impl fmt::Display for VmmMapPfnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmMapPfnType::Zero => "Zero",
            VmmMapPfnType::Free => "Free",
            VmmMapPfnType::Standby => "Standby",
            VmmMapPfnType::Modified => "Modified",
            VmmMapPfnType::ModifiedNoWrite => "ModifiedNoWrite",
            VmmMapPfnType::Bad => "Bad",
            VmmMapPfnType::Active => "Active",
            VmmMapPfnType::Transition => "Transition",
        };
        write!(f, "{v}")
    }
}

impl From<u32> for VmmMapPfnTypeExtended {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmMapPfnTypeExtended::Unused,
            2 => VmmMapPfnTypeExtended::ProcessPrivate,
            3 => VmmMapPfnTypeExtended::PageTable,
            4 => VmmMapPfnTypeExtended::LargePage,
            5 => VmmMapPfnTypeExtended::DriverLocked,
            6 => VmmMapPfnTypeExtended::Shareable,
            7 => VmmMapPfnTypeExtended::File,
            _ => VmmMapPfnTypeExtended::Unknown,
        };
    }
}

impl fmt::Display for VmmMapPfnTypeExtended {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmMapPfnTypeExtended::Unused => "Unused",
            VmmMapPfnTypeExtended::ProcessPrivate => "ProcessPrivate",
            VmmMapPfnTypeExtended::PageTable => "PageTable",
            VmmMapPfnTypeExtended::LargePage => "LargePage",
            VmmMapPfnTypeExtended::DriverLocked => "DriverLocked",
            VmmMapPfnTypeExtended::Shareable => "Shareable",
            VmmMapPfnTypeExtended::File => "File",
            VmmMapPfnTypeExtended::Unknown => "Unknown",
        };
        write!(f, "{v}")
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVmEntry {
    hVM : usize,
    uszName : *const c_char,
    gpaMax : u64,
    tp : u32,
    fActive : bool,
    fReadOnly : bool,
    fPhysicalOnly : bool,
    dwPartitionID : u32,
    dwVersionBuild : u32,
    tpSystem : u32,
    dwParentVmmMountID : u32,
    dwVmMemPID : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVmMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CVmEntry,
}

impl fmt::Display for VmmMapPfnEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapPfnEntry:{}", self.pfn)
    }
}

impl fmt::Display for VmmMapMemoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapMemoryEntry:{:x}->{:x}", self.pa, self.pa + self.cb - 1)
    }
}

impl fmt::Display for VmmMapNetEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapNetEntry:'{}'", self.desc)
    }
}

impl PartialEq for VmmMapNetEntry {
    fn eq(&self, other: &Self) -> bool {
        self.va_object == other.va_object
    }
}

impl fmt::Display for VmmMapPoolEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapPoolEntry:'{}':{:x}", self.tag_to_string(), self.va)
    }
}

impl PartialEq for VmmMapPoolEntry {
    fn eq(&self, other: &Self) -> bool {
        self.va == other.va
    }
}

impl fmt::Display for VmmMapServiceEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapServiceEntry:{}", self.name)
    }
}

impl fmt::Display for VmmMapUserEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapUserEntry:[{}]", self.user)
    }
}

impl fmt::Display for VmmMapVirtualMachineEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmMapVirtualMachineEntry:[{}]", self.name)
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct CPfnEntry {
    dwPfn : u32,
    tpExtended : u32,
    dwPfnPte : [u32; 5],
    va : u64,
    vaPte : u64,
    OriginalPte : u64,
    u3 : u32,
    u4 : u64,
    _FutureUse : [u32; 6],
}

#[repr(C)]
#[allow(non_snake_case)]
struct CPfnMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    cMap : u32,
    pMap : CPfnEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CNetMapEntry {
    dwPID : u32,
    dwState : u32,
    _FutureUse3 : [u16; 3],
    AF : u16,
    src_fValid : bool,
    src__Reserved : u16,
    src_port : u16,
    src_pbAddr : [u8; 16],
    src_uszText : *const c_char,
    dst_fValid : bool,
    dst__Reserved : u16,
    dst_port : u16,
    dst_pbAddr : [u8; 16],
    dst_uszText : *const c_char,
    vaObj : u64,
    ftTime : u64,
    dwPoolTag : u32,
    _FutureUse4 : u32,
    uszText : *const c_char,
    _FutureUse2 : [u32; 4],
}

#[repr(C)]
#[allow(non_snake_case)]
struct CNetMap {
    dwVersion : u32,
    _Reserved1 : u32,
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CNetMapEntry,
}

#[repr(C)]
struct CMemoryMapEntry {
    pa : u64,
    cb : u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CMemoryMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    cMap : u32,
    _Reserved2 : u32,
    pMap : CMemoryMapEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CPoolEntry {
    va : u64,
    dwTag : u32,
    _ReservedZero : u8,
    fAlloc : u8,
    tpPool : u8,
    tpSS : u8,
    cb : u32,
    _Filler : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CPoolMap {
    dwVersion : u32,
    _Reserved1 : [u32; 6],
    cbTotal : u32,
    piTag2Map : usize,      // ptr
    pTag : usize,           // ptr
    cTag : u32,
    cMap : u32,
    pMap : CPoolEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CServiceEntry {
    vaObj : u64,
    dwOrdinal : u32,
    dwStartType : u32,
    dwServiceType : u32,
    dwCurrentState : u32,
    dwControlsAccepted : u32,
    dwWin32ExitCode : u32,
    dwServiceSpecificExitCode : u32,
    dwCheckPoint : u32,
    wWaitHint : u32,
    uszServiceName : *const c_char,
    uszDisplayName : *const c_char,
    uszPath : *const c_char,
    uszUserTp : *const c_char,
    uszUserAcct : *const c_char,
    uszImagePath : *const c_char,
    dwPID : u32,
    _FutureUse1 : u32,
    _FutureUse2 : u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CServiceMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CServiceEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CUserEntry {
    _FutureUse1 : [u32; 2],
    uszText : *const c_char,
    vaRegHive : u64,
    uszSID : *const c_char,
    _FutureUse2 : [u32; 2],
}

#[repr(C)]
#[allow(non_snake_case)]
struct CUserMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CUserEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CRegHive {
    magic : u64,
    wVersion : u16,
    wSize : u16,
    _FutureReserved1 : [u8; 0x34],
    vaCMHIVE : u64,
    vaHBASE_BLOCK : u64,
    cbLength : u32,
    uszName : [i8; 128],
    uszNameShort : [i8; 32 + 1],
    uszHiveRootPath : [i8; 260],
    _FutureReserved : [u64; 0x10],
}

impl fmt::Display for VmmVfsEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_directory {
            write!(f, "VmmVfsEntry:D:'{}'", self.name,)
        } else {
            write!(f, "VmmVfsEntry:F:'{}':0x{:x}", self.name, self.size)
        }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVMMDLL_VFS_FILELIST2 {
    dwVersion : u32,
    pfnAddFile : extern "C" fn(h : &mut Vec<VmmVfsEntry>, uszName : *const c_char, cb : u64, pExInfo : usize),
    pfnAddDirectory : extern "C" fn(h : &mut Vec<VmmVfsEntry>, uszName : *const c_char, pExInfo : usize),
    h : *mut Vec<VmmVfsEntry>,
}

extern "C" fn vfs_list_addfile_cb(h : &mut Vec<VmmVfsEntry>, name : *const c_char, cb : u64, _p_ex_info : usize) {
    unsafe {
        if let Ok(name) = CStr::from_ptr(name).to_str() {
            let e = VmmVfsEntry {
                name : name.to_string(),
                is_directory : false,
                size : cb,
            };
            h.push(e);
        }
    }
}

extern "C" fn vfs_list_adddirectory_cb(h : &mut Vec<VmmVfsEntry>, name : *const c_char, _p_ex_info : usize) {
    unsafe {
        if let Ok(name) = CStr::from_ptr(name).to_str() {
            let e = VmmVfsEntry {
                name : name.to_string(),
                is_directory : true,
                size : 0,
            };
            h.push(e);
        }
    }
}

#[allow(non_snake_case)]
impl Vmm<'_> {
    fn impl_log(&self, log_mid : u32, log_level : &VmmLogLevel, log_message : &str) {
        let c_loglevel : u32 = match log_level {
            VmmLogLevel::_1Critical => 1,
            VmmLogLevel::_2Warning => 2,
            VmmLogLevel::_3Info => 3,
            VmmLogLevel::_4Verbose => 4,
            VmmLogLevel::_5Debug => 5,
            VmmLogLevel::_6Trace => 6,
            VmmLogLevel::_7None => 7,
        };
        let sz_log_fmt = CString::new("%s").unwrap();
        let sz_log_message = CString::new(log_message).unwrap();
        let _r = (self.native.VMMDLL_Log)(self.native.h, log_mid, c_loglevel, sz_log_fmt.as_ptr(), sz_log_message.as_ptr());
    }

    fn impl_get_config(&self, config_id : u64) -> ResultEx<u64> {
        let mut v = 0;
        let f = (self.native.VMMDLL_ConfigGet)(self.native.h, config_id, &mut v);
        return if f { Ok(v) } else { Err("VMMDLL_ConfigGet: fail".into()) };
    }

    fn impl_set_config(&self, config_id : u64, config_value : u64) -> ResultEx<()> {
        let f = (self.native.VMMDLL_ConfigSet)(self.native.h, config_id, config_value);
        return if f { Ok(()) } else { Err("VMMDLL_ConfigSet: fail".into()) };
    }

    fn impl_process_from_pid(&self, pid : u32) -> ResultEx<VmmProcess> {
        let process_list = self.process_list()?;
        let process = VmmProcess {
            vmm : &self,
            pid : pid,
        };
        if process_list.contains(&process) {
            return Ok(process);
        }
        return Err(format!("VMMDLL_PidGetFromName: fail. PID '{pid}' does not exist.").into());
    }

    fn impl_process_from_name(&self, process_name : &str) -> ResultEx<VmmProcess> {
        let mut pid = 0;
        let sz_process_name = CString::new(process_name)?;
        let r = (self.native.VMMDLL_PidGetFromName)(self.native.h, sz_process_name.as_ptr(), &mut pid);
        if !r {
            return Err(format!("VMMDLL_PidGetFromName: fail. Process '{process_name}' does not exist.").into());
        }
        return Ok(VmmProcess {
            vmm : &self,
            pid : pid,
        });
    }

    fn impl_process_list(&self) -> ResultEx<Vec<VmmProcess>> {
        let mut cpids : usize = 0;
        let r = (self.native.VMMDLL_PidList)(self.native.h, std::ptr::null_mut(), &mut cpids);
        if !r || cpids > 0x00100000 {
            return Err("VMMDLL_PidList: fail.".into());
        }
        let mut pids = vec![0u32; cpids];
        let r = (self.native.VMMDLL_PidList)(self.native.h, pids.as_mut_ptr(), &mut cpids);
        if !r || cpids > 0x00100000 {
            return Err("VMMDLL_PidList: fail.".into());
        }
        let mut proclist = Vec::new();
        for i in 0..cpids {
            let proc = VmmProcess {
                vmm : self,
                pid : *pids.get(i).unwrap(),
            };
            proclist.push(proc);
        }
        return Ok(proclist);
    }
    fn impl_map_pfn(&self, pfns : &Vec<u32>, is_extended : bool) -> ResultEx<Vec<VmmMapPfnEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let flags = if is_extended { 1 } else { 0 };
            let r = (self.native.VMMDLL_Map_GetPfnEx)(self.native.h, pfns.as_ptr(), u32::try_from(pfns.len())?, &mut structs, flags);
            if !r {
                return Err("VMMDLL_Map_GetPfnEx: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_PFN_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetPfnEx: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapPfnEntry {
                    pfn : ne.dwPfn,
                    location : VmmMapPfnType::from((ne.u3 >> 16) & 7),
                    is_prototype : if ne.u4 & 0x0200000000000000 > 0 { true } else { false },
                    color : u32::try_from(ne.u4 >> 58)?,
                    is_extended : is_extended,
                    tp_ex : VmmMapPfnTypeExtended::from(ne.tpExtended),
                    pid : ne.dwPfnPte[0],
                    ptes : [0, ne.dwPfnPte[1], ne.dwPfnPte[2], ne.dwPfnPte[3], ne.dwPfnPte[4]],
                    va : ne.va,
                    va_pte : ne.vaPte,
                    pte_original : ne.OriginalPte,
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_memory(&self) -> ResultEx<Vec<VmmMapMemoryEntry>> {
        unsafe {
            let mut structs  = std::ptr::null_mut();
            let r = (self.native.VMMDLL_Map_GetPhysMem)(self.native.h, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetPhysMem: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_PHYSMEM_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetPhysMem: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapMemoryEntry {
                    pa : ne.pa,
                    cb : ne.cb,
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_net(&self) -> ResultEx<Vec<VmmMapNetEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.native.VMMDLL_Map_GetNetU)(self.native.h, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetNetU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_NET_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetNetU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapNetEntry {
                    pid : ne.dwPID,
                    state : ne.dwState,
                    address_family : ne.AF,
                    src_is_valid : ne.src_fValid,
                    src_port : ne.src_port,
                    src_addr_raw : ne.src_pbAddr,
                    src_str : String::from(CStr::from_ptr(ne.src_uszText).to_str().unwrap_or("")),
                    dst_is_valid : ne.dst_fValid,
                    dst_port : ne.dst_port,
                    dst_addr_raw : ne.dst_pbAddr,
                    dst_str : String::from(CStr::from_ptr(ne.dst_uszText).to_str().unwrap_or("")),
                    va_object : ne.vaObj,
                    filetime : ne.ftTime,
                    pool_tag : ne.dwPoolTag,
                    desc : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_pool(&self, is_bigpool_only : bool) -> ResultEx<Vec<VmmMapPoolEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let flags = if is_bigpool_only { 1 } else { 0 };
            let r = (self.native.VMMDLL_Map_GetPool)(self.native.h, &mut structs, flags);
            if !r {
                return Err("VMMDLL_Map_GetPool: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_POOL_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetPool: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapPoolEntry {
                    va : ne.va,
                    cb : ne.cb,
                    tag : ne.dwTag,
                    is_alloc : ne.fAlloc != 0,
                    tp_pool : ne.tpPool,
                    tp_subsegment : ne.tpSS,
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_service(&self) -> ResultEx<Vec<VmmMapServiceEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.native.VMMDLL_Map_GetServicesU)(self.native.h, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetServicesU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_SERVICE_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetServicesU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapServiceEntry {
                    ordinal : ne.dwOrdinal,
                    va_object : ne.vaObj,
                    pid : ne.dwPID,
                    start_type : ne.dwStartType,
                    service_type : ne.dwServiceType,
                    current_state : ne.dwCurrentState,
                    controls_accepted : ne.dwControlsAccepted,
                    win32_exit_code : ne.dwWin32ExitCode,
                    service_specific_exit_code : ne.dwServiceSpecificExitCode,
                    check_point : ne.dwCheckPoint,
                    wait_hint : ne.wWaitHint,
                    name : String::from(CStr::from_ptr(ne.uszServiceName).to_str().unwrap_or("")),
                    name_display : String::from(CStr::from_ptr(ne.uszDisplayName).to_str().unwrap_or("")),
                    path : String::from(CStr::from_ptr(ne.uszPath).to_str().unwrap_or("")),
                    user_type : String::from(CStr::from_ptr(ne.uszUserTp).to_str().unwrap_or("")),
                    user_account : String::from(CStr::from_ptr(ne.uszUserAcct).to_str().unwrap_or("")),
                    image_path : String::from(CStr::from_ptr(ne.uszImagePath).to_str().unwrap_or("")),
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_user(&self) -> ResultEx<Vec<VmmMapUserEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.native.VMMDLL_Map_GetUsersU)(self.native.h, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetUsersU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_USER_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetUsersU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapUserEntry {
                    user : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                    sid : String::from(CStr::from_ptr(ne.uszSID).to_str().unwrap_or("")),
                    va_reg_hive : ne.vaRegHive,
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_virtual_machine(&self) -> ResultEx<Vec<VmmMapVirtualMachineEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.native.VMMDLL_Map_GetVMU)(self.native.h, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetVMU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_VM_VERSION {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetVMU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmMapVirtualMachineEntry {
                    h_vmm : self.native.h,
                    h_vm : ne.hVM,
                    name : String::from(CStr::from_ptr(ne.uszName).to_str().unwrap_or("")),
                    gpa_max : ne.gpaMax,
                    tp_vm : ne.tp,
                    is_active : ne.fActive,
                    is_readonly : ne.fReadOnly,
                    is_physicalonly : ne.fPhysicalOnly,
                    partition_id : ne.dwPartitionID,
                    guest_os_version_build : ne.dwVersionBuild,
                    guest_tp_system : ne.tpSystem,
                    parent_mount_id : ne.dwParentVmmMountID,
                    vmmem_pid : ne.dwVmMemPID,
                };
                result.push(e);
            }
            (self.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_mem_read(&self, pid : u32, va : u64, size : usize, flags : u64) -> ResultEx<Vec<u8>> {
        let cb = u32::try_from(size)?;
        let mut cb_read = 0;
        let mut pb_result = vec![0u8; size];
        let r = (self.native.VMMDLL_MemReadEx)(self.native.h, pid, va, pb_result.as_mut_ptr(), cb, &mut cb_read, flags);
        if !r {
            return Err("VMMDLL_MemReadEx: fail.".into());
        }
        return Ok(pb_result);
    }

    fn impl_mem_read_as<T>(&self, pid : u32, va : u64, flags : u64) -> ResultEx<T> {
        unsafe {
            let cb = u32::try_from(std::mem::size_of::<T>())?;
            let mut cb_read = 0;
            let mut result : T = std::mem::zeroed();
            let r = (self.native.VMMDLL_MemReadEx)(self.native.h, pid, va, &mut result as *mut _ as *mut u8, cb, &mut cb_read, flags);
            if !r {
                return Err("VMMDLL_MemReadEx: fail.".into());
            }
            return Ok(result);
        }
    }

    fn impl_mem_scatter(&self, pid : u32, flags : u64) -> ResultEx<VmmScatterMemory> {
        let flags = u32::try_from(flags)?;
        let r = (self.native.VMMDLL_Scatter_Initialize)(self.native.h, pid, flags);
        if r == 0 {
            return Err("VMMDLL_Scatter_Initialize: fail.".into());
        }
        return Ok(VmmScatterMemory {
            vmm : &self,
            hs : r,
            pid,
            flags,
            is_scatter_ex : false,
        });
    }

    fn impl_mem_virt2phys(&self, pid : u32, va : u64) -> ResultEx<u64> {
        let mut pa : u64 = 0;
        let r = (self.native.VMMDLL_MemVirt2Phys)(self.native.h, pid, va, &mut pa);
        if !r {
            return Err("VMMDLL_MemVirt2Phys: fail.".into());
        }
        return Ok(pa);
    }

    fn impl_mem_write(&self, pid : u32, va : u64, data : &Vec<u8>) -> ResultEx<()> {
        let cb = u32::try_from(data.len())?;
        let pb = data.as_ptr();
        let r = (self.native.VMMDLL_MemWrite)(self.native.h, pid, va, pb, cb);
        if !r {
            return Err("VMMDLL_MemWrite: fail.".into());
        }
        return Ok(());
    }

    fn impl_mem_write_as<T>(&self, pid : u32, va : u64, data : &T) -> ResultEx<()> {
        let cb = u32::try_from(std::mem::size_of::<T>())?;
        let r = (self.native.VMMDLL_MemWrite)(self.native.h, pid, va, data as *const _ as *const u8, cb);
        if !r {
            return Err("VMMDLL_MemWrite: fail.".into());
        }
        return Ok(());
    }

    fn impl_vfs_list(&self, path : &str) -> ResultEx<Vec<VmmVfsEntry>> {
        let c_path = CString::new(str::replace(path, "/", "\\"))?;
        let mut vec_result : Vec<VmmVfsEntry> = Vec::new();
        let ptr_result : *mut Vec<VmmVfsEntry> = &mut vec_result;
        let mut filelist2 = CVMMDLL_VFS_FILELIST2 {
            dwVersion : VMMDLL_VFS_FILELIST_VERSION,
            pfnAddFile : vfs_list_addfile_cb,
            pfnAddDirectory : vfs_list_adddirectory_cb,
            h : ptr_result,
        };
        let r = (self.native.VMMDLL_VfsListU)(self.native.h, c_path.as_ptr(), &mut filelist2);
        if !r {
            return Err("VMMDLL_VfsListU: fail.".into());
        }
        return Ok(vec_result);
    }

    fn impl_vfs_read(&self, filename : &str, size : u32, offset : u64) -> ResultEx<Vec<u8>> {
        let c_filename = CString::new(str::replace(filename, "/", "\\"))?;
        let mut cb_read = 0u32;
        let mut data = vec![0u8; size as usize];
        let ntstatus = (self.native.VMMDLL_VfsReadU)(self.native.h, c_filename.as_ptr(), data.as_mut_ptr(), size, &mut cb_read, offset);
        if ntstatus != 0 && ntstatus != 0xC0000011 {
            return Err("VMMDLL_VfsReadU: fail.".into());
        }
        if cb_read < size {
            data.resize(cb_read as usize, 0);
        }
        return Ok(data);
    }

    fn impl_vfs_write(&self, filename : &str, data : Vec<u8>, offset : u64) {
        if data.len() < u32::MAX as usize {
            let c_filename = CString::new(str::replace(filename, "/", "\\")).unwrap();
            let mut cb_write = 0u32;
            let _ntstatus = (self.native.VMMDLL_VfsWriteU)(self.native.h, c_filename.as_ptr(), data.as_ptr(), data.len() as u32, &mut cb_write, offset);
        }
    }

    fn impl_reg_hive_list(&self) -> ResultEx<Vec<VmmRegHive>> {
        unsafe {
            let mut cHives = 0;
            let r = (self.native.VMMDLL_WinReg_HiveList)(self.native.h, std::ptr::null_mut(), 0, &mut cHives);
            if !r {
                return Err("VMMDLL_WinReg_HiveList: fail.".into());
            }
            if cHives == 0 {
                return Ok(Vec::new());
            }
            let size = std::mem::size_of::<CRegHive>();
            let mut bytes = vec![0u8; size * cHives as usize];
            let ptr = bytes.as_mut_ptr() as *mut CRegHive;
            let r = (self.native.VMMDLL_WinReg_HiveList)(self.native.h, ptr, cHives, &mut cHives);
            if !r {
                return Err("VMMDLL_WinReg_HiveList: fail.".into());
            }
            if cHives == 0 {
                return Ok(Vec::new());
            }
            let mut result = Vec::new();
            let pMap = std::slice::from_raw_parts(ptr, cHives as usize);
            for i in 0..cHives as usize {
                let ne = &pMap[i];
                if (ne.magic != VMMDLL_REGISTRY_HIVE_INFORMATION_MAGIC) || (ne.wVersion != VMMDLL_REGISTRY_HIVE_INFORMATION_VERSION) {
                    return Err("Hive Bad Version.".into());
                }
                let e = VmmRegHive {
                    vmm : &self,
                    va : ne.vaCMHIVE,
                    va_baseblock : ne.vaHBASE_BLOCK,
                    size : ne.cbLength,
                    name : String::from_utf8_lossy(CStr::from_ptr(ne.uszName.as_ptr()).to_bytes()).to_string(),
                    name_short : String::from_utf8_lossy(CStr::from_ptr(ne.uszNameShort.as_ptr()).to_bytes()).to_string(),
                    path : String::from_utf8_lossy(CStr::from_ptr(ne.uszHiveRootPath.as_ptr()).to_bytes()).to_string(),
                };
                result.push(e);
            }
            return Ok(result);
        }
    }

    fn impl_reg_pathsplit(path : &str) -> ResultEx<(&str, &str)> {
        let path = path.trim_end_matches('\\');
        if let Some(split) = path.rsplit_once('\\') {
            if (split.0.len() > 0) && (split.1.len() > 0) {
                return Ok(split);
            }
        }
        return Err("[err]".into());
    }

    fn impl_reg_key(&self, path : &str) -> ResultEx<VmmRegKey> {
        let mut ftLastWrite = 0;
        let mut cch = 0;
        let c_path = CString::new(path)?;
        let r = (self.native.VMMDLL_WinReg_EnumKeyExU)(self.native.h, c_path.as_ptr(), u32::MAX, std::ptr::null_mut(), &mut cch, &mut ftLastWrite);
        if !r {
            return Err("VMMDLL_WinReg_EnumKeyExU: fail.".into());
        }
        let pathname = Vmm::impl_reg_pathsplit(path)?;
        let result = VmmRegKey {
            vmm : &self,
            name : String::from(pathname.1),
            path : String::from(path),
            ft_last_write : ftLastWrite,
        };
        return Ok(result);
    }

    fn impl_reg_value(&self, path : &str) -> ResultEx<VmmRegValue> {
        let mut raw_value = None;
        let mut raw_type = 0;
        let mut v = [0u8; 64];
        let mut raw_size = v.len() as u32;
        let c_path = CString::new(path)?;
        let r = (self.native.VMMDLL_WinReg_QueryValueExU)(self.native.h, c_path.as_ptr(), &mut raw_type, v.as_mut_ptr(), &mut raw_size);
        if !r {
            return Err("VMMDLL_WinReg_QueryValueExU: fail.".into());
        }
        if raw_size < v.len() as u32 {
            raw_value = Some(v[0..raw_size as usize].to_vec());
        } else {
            let r = (self.native.VMMDLL_WinReg_QueryValueExU)(self.native.h, c_path.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), &mut raw_size);
            if !r {
                return Err("VMMDLL_WinReg_QueryValueExU: fail.".into());
            }
        }
        let pathname = Vmm::impl_reg_pathsplit(path)?;
        let result = VmmRegValue {
            vmm : &self,
            name : String::from(pathname.1),
            path : String::from(path),
            raw_type,
            raw_size,
            raw_value,
        };
        return Ok(result);
    }
}






//=============================================================================
// INTERNAL: VMM.KERNEL:
//=============================================================================

impl fmt::Display for VmmKernel<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmKernel")
    }
}






//=============================================================================
// INTERNAL: VMM.PDB:
//=============================================================================

impl fmt::Display for VmmPdb<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmPdb:{}", self.module)
    }
}

impl VmmPdb<'_> {
    fn impl_symbol_name_from_address(&self, va_or_offset : u64) -> ResultEx<(String, u32)> {
        let c_module = CString::new(self.module.as_str())?;
        let mut c_symbol_name = [0 as c_char; MAX_PATH];
        let mut result_symbol_displacement = 0;
        let r = (self.vmm.native.VMMDLL_PdbSymbolName)(self.vmm.native.h, c_module.as_ptr(), va_or_offset, c_symbol_name.as_mut_ptr(), &mut result_symbol_displacement);
        if !r {
            return Err("VMMDLL_PdbSymbolName: fail.".into());
        }
        let cstr_symbol_name = unsafe { CStr::from_ptr(c_symbol_name.as_ptr()) };
        let string_symbol_name = String::from_utf8_lossy(cstr_symbol_name.to_bytes()).to_string();
        return Ok((string_symbol_name, result_symbol_displacement));
    }

    fn impl_symbol_address_from_name(&self, symbol_name : &str) -> ResultEx<u64> {
        let c_module = CString::new(self.module.as_str())?;
        let c_symbol_name = CString::new(symbol_name)?;
        let mut result = 0;
        let r = (self.vmm.native.VMMDLL_PdbSymbolAddress)(self.vmm.native.h, c_module.as_ptr(), c_symbol_name.as_ptr(), &mut result);
        if !r {
            return Err("VMMDLL_PdbSymbolAddress: fail.".into());
        }
        return Ok(result);
    }

    fn impl_type_size(&self, type_name : &str) -> ResultEx<u32> {
        let c_module = CString::new(self.module.as_str())?;
        let c_type_name = CString::new(type_name)?;
        let mut result = 0;
        let r = (self.vmm.native.VMMDLL_PdbTypeSize)(self.vmm.native.h, c_module.as_ptr(), c_type_name.as_ptr(), &mut result);
        if !r {
            return Err("VMMDLL_PdbTypeSize: fail.".into());
        }
        return Ok(result);
    }

    fn impl_type_child_offset(&self, type_name : &str, type_child_name : &str) -> ResultEx<u32> {
        let c_module = CString::new(self.module.as_str())?;
        let c_type_name = CString::new(type_name)?;
        let c_type_child_name = CString::new(type_child_name)?;
        let mut result = 0;
        let r = (self.vmm.native.VMMDLL_PdbTypeChildOffset)(self.vmm.native.h, c_module.as_ptr(), c_type_name.as_ptr(), c_type_child_name.as_ptr(), &mut result);
        if !r {
            return Err("VMMDLL_PdbTypeChildOffset: fail.".into());
        }
        return Ok(result);
    }
}






//=============================================================================
// INTERNAL: VMM.REGISTRY:
//=============================================================================

impl fmt::Display for VmmRegHive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmRegHive:{:x}", self.va)
    }
}

impl PartialEq for VmmRegHive<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.va == other.va
    }
}

impl fmt::Display for VmmRegKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmRegKey:{}", self.name)
    }
}

impl PartialEq for VmmRegKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl fmt::Display for VmmRegValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmRegValueType::REG_NONE => "REG_NONE".to_string(),
            VmmRegValueType::REG_SZ(r) => format!("REG_SZ({r})"),
            VmmRegValueType::REG_EXPAND_SZ(_) => "REG_EXPAND_SZ".to_string(),
            VmmRegValueType::REG_BINARY(_) => "REG_BINARY".to_string(),
            VmmRegValueType::REG_DWORD(r) => format!("REG_DWORD(0x{:x})", r),
            VmmRegValueType::REG_DWORD_BIG_ENDIAN(r) => format!("REG_DWORD_BIG_ENDIAN(0x{:x})", r),
            VmmRegValueType::REG_LINK(r) => format!("REG_LINK({r})"),
            VmmRegValueType::REG_MULTI_SZ(_) => "REG_MULTI_SZ".to_string(),
            VmmRegValueType::REG_RESOURCE_LIST(_) => "REG_RESOURCE_LIST".to_string(),
            VmmRegValueType::REG_FULL_RESOURCE_DESCRIPTOR(_) => "REG_FULL_RESOURCE_DESCRIPTOR".to_string(),
            VmmRegValueType::REG_RESOURCE_REQUIREMENTS_LIST(_) => "REG_RESOURCE_REQUIREMENTS_LIST".to_string(),
            VmmRegValueType::REG_QWORD(r) => format!("REG_QWORD(0x{:x})", r),
        };
        write!(f, "{v}")
    }
}

impl fmt::Display for VmmRegValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmRegValue:{}", self.name)
    }
}

impl PartialEq for VmmRegValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl VmmRegHive<'_> {
    fn impl_reg_hive_read(&self, ra : u32, size : usize, flags : u64) -> ResultEx<Vec<u8>> {
        let cb = u32::try_from(size)?;
        let mut cb_read = 0;
        let mut pb_result = vec![0u8; size];
        let r = (self.vmm.native.VMMDLL_WinReg_HiveReadEx)(self.vmm.native.h, self.va, ra, pb_result.as_mut_ptr(), cb, &mut cb_read, flags);
        if !r {
            return Err("VMMDLL_WinReg_HiveReadEx: fail.".into());
        }
        return Ok(pb_result);
    }

    fn impl_reg_hive_write(&self, ra : u32, data : &Vec<u8>) -> ResultEx<()> {
        let cb = u32::try_from(data.len())?;
        let pb = data.as_ptr();
        let r = (self.vmm.native.VMMDLL_WinReg_HiveWrite)(self.vmm.native.h, self.va, ra, pb, cb);
        if !r {
            return Err("VMMDLL_WinReg_HiveWrite: fail.".into());
        }
        return Ok(());
    }
}

impl VmmRegKey<'_> {
    fn impl_parent(&self) -> ResultEx<VmmRegKey> {        
        let pathfile = Vmm::impl_reg_pathsplit(self.path.as_str())?;
        let result = self.vmm.impl_reg_key(pathfile.0)?;
        return Ok(result);
    }

    #[allow(unused_assignments)]
    fn impl_subkeys(&self) -> ResultEx<Vec<VmmRegKey>> {
        unsafe {
            let mut ft_last_write = 0;
            let mut cch = 0;
            let mut i = 0;
            let mut data = [0; MAX_PATH+1];
            let c_path = CString::new(self.path.as_str())?;
            let mut result = Vec::new();
            loop {
                cch = data.len() as u32 - 1;
                let r = (self.vmm.native.VMMDLL_WinReg_EnumKeyExU)(self.vmm.native.h, c_path.as_ptr(), i, data.as_mut_ptr(), &mut cch, &mut ft_last_write);
                if !r {
                    break;
                }
                let name = String::from_utf8_lossy(CStr::from_ptr(data.as_ptr()).to_bytes()).to_string();
                let path = format!("{}\\{}", self.path, name);
                let e = VmmRegKey {
                    vmm : self.vmm,
                    name,
                    path,
                    ft_last_write,
                };
                result.push(e);
                i += 1;
            }
            return Ok(result);
        }
    }

    fn impl_values(&self) -> ResultEx<Vec<VmmRegValue>> {
        return Err("Not implemented".into());
    }
}

impl VmmRegValue<'_> {
    fn impl_parent(&self) -> ResultEx<VmmRegKey> {        
        let pathfile = Vmm::impl_reg_pathsplit(self.path.as_str())?;
        let result = self.vmm.impl_reg_key(pathfile.0)?;
        return Ok(result);
    }

    fn impl_raw_value(&self) -> ResultEx<Vec<u8>> {
            if self.raw_value.is_some() {
                return Ok(self.raw_value.as_ref().unwrap().clone());
            }
            // size larger than 64 bytes -> not cached in VmmRegValue.
            if self.raw_size > 0x01000000 {
                return Err("VmmRegKey size too large (>16MB).".into());
            }
            let mut raw_value = vec![0; self.raw_size as usize];
            let c_path = CString::new(self.path.clone())?;
            let mut raw_size = self.raw_size;
            let r = (self.vmm.native.VMMDLL_WinReg_QueryValueExU)(self.vmm.native.h, c_path.as_ptr(), std::ptr::null_mut(), raw_value.as_mut_ptr(), &mut raw_size);
            if !r {
                return Err("VMMDLL_WinReg_QueryValueExU: fail.".into());
            }
            return Ok(raw_value);
    }

    fn impl_value(&self) -> ResultEx<VmmRegValueType> {
        const REG_NONE                      : u32 = 0;
        const REG_SZ                        : u32 = 1;
        const REG_EXPAND_SZ                 : u32 = 2;
        const REG_BINARY                    : u32 = 3;
        const REG_DWORD                     : u32 = 4;
        const REG_DWORD_BIG_ENDIAN          : u32 = 5;
        const REG_LINK                      : u32 = 6;
        const REG_MULTI_SZ                  : u32 = 7;
        const REG_RESOURCE_LIST             : u32 = 8;
        const REG_FULL_RESOURCE_DESCRIPTOR  : u32 = 9;
        const REG_RESOURCE_REQUIREMENTS_LIST: u32 = 10;
        const REG_QWORD                     : u32 = 11;
        // Sanity checks and REG_NONE type:
        if self.raw_type == REG_NONE {
            return Ok(VmmRegValueType::REG_NONE);
        }
        if self.raw_type > REG_QWORD {
            return Err("Unknown registry value type.".into());
        }
        // Get data using method call since data may be larger than cached data.
        let raw_value = self.raw_value()?;
        match self.raw_type {
            REG_BINARY => return Ok(VmmRegValueType::REG_BINARY(raw_value)),
            REG_RESOURCE_LIST => return Ok(VmmRegValueType::REG_RESOURCE_LIST(raw_value)),
            REG_FULL_RESOURCE_DESCRIPTOR => return Ok(VmmRegValueType::REG_FULL_RESOURCE_DESCRIPTOR(raw_value)),
            REG_RESOURCE_REQUIREMENTS_LIST => return Ok(VmmRegValueType::REG_RESOURCE_REQUIREMENTS_LIST(raw_value)),
            _ => (),
        };
        if self.raw_type == REG_DWORD {
            let v : [u8; 4] = raw_value.as_slice().try_into()?;
            return Ok(VmmRegValueType::REG_DWORD(u32::from_le_bytes(v)));
        }
        if self.raw_type == REG_DWORD_BIG_ENDIAN {
            let v : [u8; 4] = raw_value.as_slice().try_into()?;
            return Ok(VmmRegValueType::REG_DWORD_BIG_ENDIAN(u32::from_be_bytes(v)));
        }
        if self.raw_type == REG_QWORD {
            let v : [u8; 8] = raw_value.as_slice().try_into()?;
            return Ok(VmmRegValueType::REG_QWORD(u64::from_le_bytes(v)));
        }
        // UTF16 below
        if raw_value.len() % 2 == 1 {
            return Err("Invalid size".into());
        }
        let mut raw_chars = vec![0u16; raw_value.len() / 2];
        unsafe {
            // this will only work on little-endian archs (which should be most)
            std::ptr::copy_nonoverlapping(raw_value.as_ptr(), raw_chars.as_mut_ptr() as *mut u8, raw_value.len());
        }
        if self.raw_type == REG_MULTI_SZ {
            let mut result_vec = Vec::new();
            for raw_string in raw_chars.split(|v| *v == 0) {
                if raw_string.len() > 0 {
                    result_vec.push(String::from_utf16_lossy(raw_string));
                }
            }
            return Ok(VmmRegValueType::REG_MULTI_SZ(result_vec));
        }
        // SZ EXPAND_SZ, LINK
        let mut result_string = "".to_string();
        if let Some(raw_string) = raw_chars.split(|v| *v == 0).next() {
            result_string = String::from_utf16_lossy(raw_string);
        }
        match self.raw_type {
            REG_SZ => return Ok(VmmRegValueType::REG_SZ(result_string)),
            REG_EXPAND_SZ => return Ok(VmmRegValueType::REG_EXPAND_SZ(result_string)),
            REG_LINK => return Ok(VmmRegValueType::REG_LINK(result_string)),
            _ => return Err("[err]".into()),
        };
    }
}






//=============================================================================
// INTERNAL: VMM.PROCESS:
//=============================================================================

impl fmt::Display for VmmProcess<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcess:{}", self.pid)
    }
}

impl PartialEq for VmmProcess<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}

impl fmt::Display for VmmProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessInfo:{}:{}", self.pid, self.name)
    }
}

impl fmt::Display for VmmProcessMapEatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapEatEntry:{:x}:{}", self.va_function, self.function)
    }
}

impl fmt::Display for VmmProcessMapHandleEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapHandleEntry:{}:{:x}:{}:[{}]", self.pid, self.handle_id, self.tp, self.info)
    }
}

impl From<u32> for VmmProcessMapHeapType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmProcessMapHeapType::NtHeap,
            2 => VmmProcessMapHeapType::SegmentHeap,
            _ => VmmProcessMapHeapType::NA,
        };
    }
}

impl fmt::Display for VmmProcessMapHeapType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmProcessMapHeapType::NA => "NA",
            VmmProcessMapHeapType::NtHeap => "NtHeap",
            VmmProcessMapHeapType::SegmentHeap => "SegmentHeap",
        };
        write!(f, "{v}")
    }
}

impl fmt::Display for VmmProcessMapHeapEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapHeapAllocEntry:{}:{}:{}", self.pid, self.number, self.tp)
    }
}

impl From<u32> for VmmProcessMapHeapAllocType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmProcessMapHeapAllocType::NtHeap,
            2 => VmmProcessMapHeapAllocType::NtLFH,
            3 => VmmProcessMapHeapAllocType::NtLarge,
            4 => VmmProcessMapHeapAllocType::NtNA,
            5 => VmmProcessMapHeapAllocType::SegVS,
            6 => VmmProcessMapHeapAllocType::SegLFH,
            7 => VmmProcessMapHeapAllocType::SegLarge,
            8 => VmmProcessMapHeapAllocType::SegNA,
            _ => VmmProcessMapHeapAllocType::NA,
        };
    }
}

impl fmt::Display for VmmProcessMapHeapAllocType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmProcessMapHeapAllocType::NA => "NA",
            VmmProcessMapHeapAllocType::NtHeap => "NtHeap",
            VmmProcessMapHeapAllocType::NtLFH => "NtLFH",
            VmmProcessMapHeapAllocType::NtLarge => "NtLarge",
            VmmProcessMapHeapAllocType::NtNA => "NtNA",
            VmmProcessMapHeapAllocType::SegVS => "SegVS",
            VmmProcessMapHeapAllocType::SegLFH => "SegLFH",
            VmmProcessMapHeapAllocType::SegLarge => "SegLarge",
            VmmProcessMapHeapAllocType::SegNA => "SegNA",
        };
        write!(f, "{v}")
    }
}

impl fmt::Display for VmmProcessMapHeapAllocEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapHeapAllocEntry:{}:{}:{:x}", self.pid, self.tp, self.va)
    }
}

impl fmt::Display for VmmProcessMapIatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapIatEntry:{:x}:{}", self.va_function, self.function)
    }
}

impl fmt::Display for VmmProcessMapPteEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapPteEntry:{}:{:x}->{:x}", self.pid, self.va_base, self.va_base + self.page_count * 0x1000 - 1)
    }
}

impl fmt::Display for VmmProcessMapModuleEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapModuleEntry:{}:{:x}:[{}]", self.pid, self.va_base, self.name)
    }
}

impl fmt::Display for VmmProcessMapModuleDebugEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapModuleDebugEntry:[{}]", self.pdb_filename)
    }
}

impl fmt::Display for VmmProcessMapModuleVersionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapModuleVersionEntry")
    }
}

impl fmt::Display for VmmProcessMapThreadEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapThreadEntry:{}:{:x}", self.pid, self.thread_id)
    }
}

impl fmt::Display for VmmProcessMapUnloadedModuleEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapUnloadedModuleEntry:{}:{:x}:[{}]", self.pid, self.va_base, self.name)
    }
}

impl fmt::Display for VmmProcessMapVadEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapVadEntry:{}:{:x}->{}", self.pid, self.va_start, self.va_end)
    }
}

impl From<u32> for VmmProcessMapVadExType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmProcessMapVadExType::Hardware,
            2 => VmmProcessMapVadExType::Transition,
            3 => VmmProcessMapVadExType::Prototype,
            4 => VmmProcessMapVadExType::DemandZero,
            5 => VmmProcessMapVadExType::Compressed,
            6 => VmmProcessMapVadExType::Pagefile,
            7 => VmmProcessMapVadExType::File,
            _ => VmmProcessMapVadExType::NA,
        };
    }
}

impl fmt::Display for VmmProcessMapVadExType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmProcessMapVadExType::NA => "NA",
            VmmProcessMapVadExType::Hardware => "Hardware",
            VmmProcessMapVadExType::Transition => "Transition",
            VmmProcessMapVadExType::Prototype => "Prototype",
            VmmProcessMapVadExType::DemandZero => "DemandZero",
            VmmProcessMapVadExType::Compressed => "Compressed",
            VmmProcessMapVadExType::Pagefile => "Pagefile",
            VmmProcessMapVadExType::File => "File",
        };
        write!(f, "{v}")
    }
}

impl fmt::Display for VmmProcessMapVadExEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapVadExEntry:{}:{:x}:{}", self.pid, self.va, self.tp)
    }
}

impl fmt::Display for VmmProcessMapDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessMapDirectoryEntry:{}:{}:{:x}:{:x}", self.pid, self.name, self.virtual_address, self.size)
    }
}

impl fmt::Display for VmmProcessSectionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmProcessSectionEntry:{}:[{}]:{:x}:{:x}", self.pid, self.name, self.virtual_address, self.misc_virtual_size)
    }
}

impl From<u32> for VmmProcessMapModuleType {
    fn from(v : u32) -> Self {
        return match v {
            1 => VmmProcessMapModuleType::Data,
            2 => VmmProcessMapModuleType::NotLinked,
            3 => VmmProcessMapModuleType::Injected,
            _ => VmmProcessMapModuleType::Normal,
        };
    }
}

impl fmt::Display for VmmProcessMapModuleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            VmmProcessMapModuleType::Data => "Data",
            VmmProcessMapModuleType::NotLinked => "NotLinked",
            VmmProcessMapModuleType::Injected => "Injected",
            VmmProcessMapModuleType::Normal => "Normal",
        };
        write!(f, "{v}")
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct CProcessInformation {
    magic : u64,
    wVersion : u16,
    wSize : u16,
    tpMemoryModel : u32,
    tpSystem : u32,
    fUserOnly : bool,
    dwPID : u32,
    dwPPID : u32,
    dwState : u32,
    szName : [i8; 16],
    szNameLong : [i8; 64],
    paDTB : u64,
    paDTB_UserOpt : u64,
    vaEPROCESS : u64,
    vaPEB : u64,
    _Reserved1 : u64,
    fWow64 : bool,
    vaPEB32 : u32,
    dwSessionId : u32,
    qwLUID : u64,
    szSID : [i8; 260],
    IntegrityLevel : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Clone, Default)]
struct CIMAGE_SECTION_HEADER {
    Name : [u8; 8],
    Misc_VirtualAddress : u32,
    VirtualAddress : u32,
    SizeOfRawData : u32,
    PointerToRawData : u32,
    PointerToRelocations : u32,
    PointerToLinenumbers : u32,
    NumberOfRelocations : u16,
    NumberOfLinenumbers : u16,
    Characteristics : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Clone, Default)]
struct CIMAGE_DATA_DIRECTORY {
    VirtualAddress : u32,
    Size : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CEatEntry {
    vaFunction : u64,
    dwOrdinal : u32,
    oFunctionsArray : u32,
    oNamesArray : u32,
    _FutureUse1 : u32,
    uszFunction : *const c_char,
    uszForwardedFunction : *const c_char,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CEatMap {
    dwVersion : u32,
    dwOrdinalBase : u32,
    cNumberOfNames : u32,
    cNumberOfFunctions : u32,
    cNumberOfForwardedFunctions : u32,
    _Reserved1 : [u32; 3],
    vaModuleBase : u64,
    vaAddressOfFunctions : u64,
    vaAddressOfNames : u64,
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CEatEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CHandleEntry {
    vaObject : u64,
    dwHandle : u32,
    dwGrantedAccess_Tp : u32,
    qwHandleCount : u64,
    qwPointerCount : u64,
    vaObjectCreateInfo : u64,
    vaSecurityDescriptor : u64,
    uszText : *const c_char,
    _FutureUse2 : u32,
    dwPID : u32,
    dwPoolTag : u32,
    _FutureUse : [u32; 7],
    uszType : *const c_char,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CHandleMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CHandleEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CHeapEntry {
    va : u64,
    tp : u32,
    f32 : bool,
    iHeap : u32,
    dwHeapNum : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CHeapMap {
    dwVersion : u32,
    _Reserved1 : [u32; 7],
    pSegments : usize,
    cSegments : u32,
    cMap : u32,
    pMap : CHeapEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CHeapAllocEntry {
    va : u64,
    cb : u32,
    tp : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CHeapAllocMap {
    dwVersion : u32,
    _Reserved1 : [u32; 7],
    _Reserved2 : [usize; 2],
    cMap : u32,
    pMap : CHeapAllocEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CIatEntry {
    vaFunction : u64,
    uszFunction : *const c_char,
    _FutureUse1 : u32,
    _FutureUse2 : u32,
    uszModule : *const c_char,
    thunk_f32 : bool,
    thunk_wHint : u16,
    thunk__Reserved1 : u16,
    thunk_rvaFirstThunk : u32,
    thunk_rvaOriginalFirstThunk : u32,
    thunk_rvaNameModule : u32,
    thunk_rvaNameFunction : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CIatMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    vaModuleBase : u64,
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CIatEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CDebugInfo {
    dwAge : u32,
    _Reserved : u32,
    Guid : [u8; 16],
    uszGuid : *const c_char,
    uszPdbFilename : *const c_char,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVersionInfo {
    uszCompanyName : *const c_char,
    uszFileDescription : *const c_char,
    uszFileVersion : *const c_char,
    uszInternalName : *const c_char,
    uszLegalCopyright : *const c_char,
    uszOriginalFilename : *const c_char,
    uszProductName : *const c_char,
    uszProductVersion : *const c_char,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CModuleEntry {
    vaBase : u64,
    vaEntry : u64,
    cbImageSize : u32,
    fWoW64 : bool,
    uszText : *const c_char,
    _Reserved3 : u32,
    _Reserved4 : u32,
    uszFullName : *const c_char,
    tp : u32,
    cbFileSizeRaw : u32,
    cSection : u32,
    cEAT : u32,
    cIAT : u32,
    _Reserved2 : u32,
    _Reserved1 : [u64; 3],
    pExDebugInfo : *const CDebugInfo,
    pExVersionInfo : *const CVersionInfo,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CModuleMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CModuleEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CPteEntry {
    vaBase : u64,
    cPages : u64,
    fPage : u64,
    fWoW64 : bool,
    _FutureUse1 : u32,
    uszText : *const c_char,
    _Reserved1 : u32,
    cSoftware : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CPteMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CPteEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CThreadEntry {
    dwTID : u32,
    dwPID : u32,
    dwExitStatus : u32,
    bState : u8,
    bRunning : u8,
    bPriority : u8,
    bBasePriority : u8,
    vaETHREAD : u64,
    vaTeb : u64,
    ftCreateTime : u64,
    ftExitTime : u64,
    vaStartAddress : u64,
    vaStackBaseUser : u64,
    vaStackLimitUser : u64,
    vaStackBaseKernel : u64,
    vaStackLimitKernel : u64,
    vaTrapFrame : u64,
    vaRIP : u64,
    vaRSP : u64,
    qwAffinity : u64,
    dwUserTime : u32,
    dwKernelTime : u32,
    bSuspendCount : u8,
    bWaitReason : u8,
    _FutureUse1 : [u8; 2],
    _FutureUse2 : [u32; 13],
    vaWin32StartAddress : u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CThreadMap {
    dwVersion : u32,
    _Reserved1 : [u32; 8],
    cMap : u32,
    pMap : CThreadEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CUnloadedModuleEntry {
    vaBase : u64,
    cbImageSize : u32,
    fWoW64 : bool,
    uszText : *const c_char,
    _FutureUse1 : u32,
    dwCheckSum : u32,
    dwTimeDateStamp : u32,
    _Reserved1 : u32,
    ftUnload : u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CUnloadedModuleMap {
    dwVersion : u32,
    _Reserved1 : [u32; 5],
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CUnloadedModuleEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVadEntry {
    vaStart : u64,
    vaEnd : u64,
    vaVad : u64,
    u0 : u32,
    u1 : u32,
    u2 : u32,
    cbPrototypePte : u32,
    vaPrototypePte : u64,
    vaSubsection : u64,
    uszText : *const c_char,
    _FutureUse1 : u32,
    _Reserved1 : u32,
    vaFileObject : u64,
    cVadExPages : u32,
    cVadExPagesBase : u32,
    _Reserved2 : u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVadMap {
    dwVersion : u32,
    _Reserved1 : [u32; 4],
    cPage : u32,
    pbMultiText : *const c_char,
    cbMultiText : u32,
    cMap : u32,
    pMap : CVadEntry,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVadExEntry {
    tp : u32,
    iPML : u32,
    va : u64,
    pa : u64,
    pte : u64,
    proto__Reserved1 : u32,
    proto_tp : u32,
    proto_pa : u64,
    proto_va : u64,
    vaVadBase : u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVadExMap {
    dwVersion : u32,
    _Reserved1 : [u32; 4],
    cMap : u32,
    pMap : CVadExEntry,
}

#[allow(non_snake_case)]
impl VmmProcess<'_> {
    fn impl_info(&self) -> ResultEx<VmmProcessInfo> {
        let mut cb_pi = std::mem::size_of::<CProcessInformation>();
        let mut pi = CProcessInformation {
            magic : VMMDLL_PROCESS_INFORMATION_MAGIC,
            wVersion : VMMDLL_PROCESS_INFORMATION_VERSION,
            wSize : u16::try_from(cb_pi)?,
            tpMemoryModel : 0,
            tpSystem : 0,
            fUserOnly : false,
            dwPID : 0,
            dwPPID : 0,
            dwState : 0,
            szName : [0i8; 16],
            szNameLong : [0i8; 64],
            paDTB : 0,
            paDTB_UserOpt : 0,
            vaEPROCESS : 0,
            vaPEB : 0,
            _Reserved1 : 0,
            fWow64 : false,
            vaPEB32 : 0,
            dwSessionId : 0,
            qwLUID : 0,
            szSID : [0i8; 260],
            IntegrityLevel : 0,
        };
        let raw_pi = &mut pi as *mut CProcessInformation;
        let r = (self.vmm.native.VMMDLL_ProcessGetInformation)(self.vmm.native.h, self.pid, raw_pi, &mut cb_pi);
        if !r {
            return Err("VMMDLL_ProcessGetInformation: fail.".into());
        }
        let result = VmmProcessInfo {
            tp_system : VmmSystemType::from(pi.tpSystem),
            tp_memorymodel : VmmMemoryModelType::from(pi.tpMemoryModel),
            is_user_mode : pi.fUserOnly,
            pid : pi.dwPID,
            ppid : pi.dwPPID,
            state : pi.dwState,
            name : unsafe { CStr::from_ptr(&pi.szName as *const c_char).to_string_lossy().to_string() },
            name_long : unsafe { CStr::from_ptr(&pi.szNameLong as *const c_char).to_string_lossy().to_string() },
            pa_dtb : pi.paDTB,
            pa_dtb_user : pi.paDTB_UserOpt,
            va_eprocess : pi.vaEPROCESS,
            va_peb : pi.vaPEB,
            is_wow64 : pi.fWow64,
            va_peb32 : pi.vaPEB32,
            session_id : pi.dwSessionId,
            luid : pi.qwLUID,
            sid : unsafe { CStr::from_ptr(&pi.szSID as *const c_char).to_string_lossy().to_string() },
            integrity_level : VmmIntegrityLevelType::from(pi.IntegrityLevel),
        };
        return Ok(result);
    }

    fn impl_get_information_string(&self, option : u32) -> ResultEx<String> {
        let r = (self.vmm.native.VMMDLL_ProcessGetInformationString)(self.vmm.native.h, self.pid, option);
        if r.is_null() {
            return Err("VMMDLL_ProcessGetInformationString: fail.".into());
        }
        let cstr = unsafe { CStr::from_ptr(r) };
        let result = cstr.to_string_lossy().to_string();
        (self.vmm.native.VMMDLL_MemFree)(r as usize);
        return Ok(result);
    }
    
    fn impl_get_module_base(&self, module_name : &str) -> ResultEx<u64> {
        let sz_module_name = CString::new(module_name)?;
        let r = (self.vmm.native.VMMDLL_ProcessGetModuleBaseU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr());
        if r == 0 {
            return Err("VMMDLL_ProcessGetModuleBaseU: fail.".into());
        }
        return Ok(r);
    }

    fn impl_get_proc_address(&self, module_name : &str, function_name : &str) -> ResultEx<u64> {
        let sz_module_name = CString::new(module_name)?;
        let sz_function_name = CString::new(function_name)?;
        let r = (self.vmm.native.VMMDLL_ProcessGetProcAddressU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr(), sz_function_name.as_ptr());
        if r == 0 {
            return Err("VMMDLL_ProcessGetProcAddressU: fail.".into());
        }
        return Ok(r);
    }

    fn impl_get_proc_address_pid(&self, pid: u32, module_name : &str, function_name : &str) -> ResultEx<u64> {
        let sz_module_name = CString::new(module_name)?;
        let sz_function_name = CString::new(function_name)?;
        let r = (self.vmm.native.VMMDLL_ProcessGetProcAddressU)(self.vmm.native.h, pid, sz_module_name.as_ptr(), sz_function_name.as_ptr());
        if r == 0 {
            return Err("VMMDLL_ProcessGetProcAddressU: fail.".into());
        }
        return Ok(r);
    }

    fn impl_pdb_from_module_address(&self, va_module_base : u64) -> ResultEx<VmmPdb> {
        let mut szModuleName = [0i8; MAX_PATH + 1];
        let r = (self.vmm.native.VMMDLL_PdbLoad)(self.vmm.native.h, self.pid, va_module_base, szModuleName.as_mut_ptr());
        if !r {
            return Err("VMMDLL_PdbLoad: fail.".into());
        }
        let cstr = unsafe { CStr::from_ptr(szModuleName.as_ptr()) };
        let module = cstr.to_string_lossy().to_string();
        let pdb = VmmPdb {
            vmm : self.vmm,
            module,
        };
        return Ok(pdb);
    }

    fn impl_map_handle(&self) -> ResultEx<Vec<VmmProcessMapHandleEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetHandleU)(self.vmm.native.h, self.pid, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetHandleU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_HANDLE_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetHandleU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapHandleEntry {
                    pid : self.pid,
                    va_object : ne.vaObject,
                    handle_id : ne.dwHandle,
                    granted_access : ne.dwGrantedAccess_Tp & 0x00ffffff,
                    type_index : (ne.dwGrantedAccess_Tp >> 24) & 0xff,
                    handle_count : ne.qwHandleCount,
                    pointer_count : ne.qwPointerCount,
                    va_object_create_info : ne.vaObjectCreateInfo,
                    va_security_descriptor : ne.vaSecurityDescriptor,
                    handle_pid : ne.dwPID,
                    pool_tag : ne.dwPoolTag,
                    info : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                    tp : String::from(CStr::from_ptr(ne.uszType).to_str().unwrap_or("")),
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_heap(&self) -> ResultEx<Vec<VmmProcessMapHeapEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetHeap)(self.vmm.native.h, self.pid, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetHeap: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_HEAP_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetHeap: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapHeapEntry {
                    pid : self.pid,
                    tp : VmmProcessMapHeapType::from(ne.tp),
                    is_32 : ne.f32,
                    index : ne.iHeap,
                    number : ne.dwHeapNum,
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_heapalloc(&self, heap_number_or_address : u64) -> ResultEx<Vec<VmmProcessMapHeapAllocEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetHeapAlloc)(self.vmm.native.h, self.pid, heap_number_or_address, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetHeapAlloc: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_HEAPALLOC_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetHeapAlloc: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapHeapAllocEntry {
                    pid : self.pid,
                    va : ne.va,
                    size : ne.cb,
                    tp : VmmProcessMapHeapAllocType::from(ne.tp),
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_module(&self, is_info_debug : bool, is_info_version : bool) -> ResultEx<Vec<VmmProcessMapModuleEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let flags = 0 + if is_info_debug { 1 } else { 0 } + if is_info_version { 2 } else { 0 };
            let r = (self.vmm.native.VMMDLL_Map_GetModuleU)(self.vmm.native.h, self.pid, &mut structs, flags);
            if !r {
                return Err("VMMDLL_Map_GetModuleU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_MODULE_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetModuleU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let mut debug_info = None;
                if !ne.pExDebugInfo.is_null() {
                    let nei = &*ne.pExDebugInfo;
                    debug_info = Some(VmmProcessMapModuleDebugEntry {
                        pid : self.pid,
                        age : nei.dwAge,
                        raw_guid : nei.Guid,
                        guid : String::from(CStr::from_ptr(nei.uszGuid).to_str().unwrap_or("")),
                        pdb_filename : String::from(CStr::from_ptr(nei.uszPdbFilename).to_str().unwrap_or("")),
                    });
                }
                let mut version_info = None;
                if !ne.pExVersionInfo.is_null() {
                    let nei = &*ne.pExVersionInfo;
                    version_info = Some(VmmProcessMapModuleVersionEntry {
                        pid : self.pid,
                        company_name : String::from(CStr::from_ptr(nei.uszCompanyName).to_str().unwrap_or("")),
                        file_description : String::from(CStr::from_ptr(nei.uszFileDescription).to_str().unwrap_or("")),
                        file_version : String::from(CStr::from_ptr(nei.uszFileVersion).to_str().unwrap_or("")),
                        internal_name : String::from(CStr::from_ptr(nei.uszInternalName).to_str().unwrap_or("")),
                        legal_copyright : String::from(CStr::from_ptr(nei.uszLegalCopyright).to_str().unwrap_or("")),
                        original_file_name : String::from(CStr::from_ptr(nei.uszOriginalFilename).to_str().unwrap_or("")),
                        product_name : String::from(CStr::from_ptr(nei.uszProductName).to_str().unwrap_or("")),
                        product_version : String::from(CStr::from_ptr(nei.uszProductVersion).to_str().unwrap_or("")),
                    });
                }
                let e = VmmProcessMapModuleEntry {
                    pid : self.pid,
                    va_base : ne.vaBase,
                    va_entry : ne.vaEntry,
                    image_size : ne.cbImageSize,
                    is_wow64 : ne.fWoW64,
                    tp : VmmProcessMapModuleType::from(ne.tp),
                    name : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                    full_name : String::from(CStr::from_ptr(ne.uszFullName).to_str().unwrap_or("")),
                    file_size_raw : ne.cbFileSizeRaw,
                    section_count : ne.cSection,
                    eat_count : ne.cEAT,
                    iat_count : ne.cIAT,
                    debug_info : debug_info,
                    version_info : version_info,
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_module_eat(&self, module_name : &str) -> ResultEx<Vec<VmmProcessMapEatEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let sz_module_name = CString::new(module_name)?;
            let r = (self.vmm.native.VMMDLL_Map_GetEATU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr(), &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetEATU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_EAT_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetEATU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapEatEntry {
                    pid : self.pid,
                    va_function : ne.vaFunction,
                    ordinal : ne.dwOrdinal,
                    function : String::from(CStr::from_ptr(ne.uszFunction).to_str().unwrap_or("")),
                    forwarded_function : String::from(CStr::from_ptr(ne.uszForwardedFunction).to_str().unwrap_or("")),
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_module_iat(&self, module_name : &str) -> ResultEx<Vec<VmmProcessMapIatEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let sz_module_name = CString::new(module_name)?;
            let r = (self.vmm.native.VMMDLL_Map_GetIATU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr(), &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetIATU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_IAT_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetIATU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapIatEntry {
                    pid : self.pid,
                    va_function : ne.vaFunction,
                    function : String::from(CStr::from_ptr(ne.uszFunction).to_str().unwrap_or("")),
                    module : String::from(CStr::from_ptr(ne.uszModule).to_str().unwrap_or("")),
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_pte(&self, is_identify_modules : bool) -> ResultEx<Vec<VmmProcessMapPteEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetPteU)(self.vmm.native.h, self.pid, is_identify_modules, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetPteU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_PTE_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetPteU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapPteEntry {
                    pid : self.pid,
                    va_base : ne.vaBase,
                    page_count : ne.cPages,
                    page_software_count : ne.cSoftware,
                    is_r : true,
                    is_w : (ne.fPage & 0x0000000000000002) != 0,
                    is_x : (ne.fPage & 0x8000000000000000) == 0,
                    is_s : (ne.fPage & 0x0000000000000004) == 0,
                    is_wow64 : ne.fWoW64,
                    info : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_thread(&self) -> ResultEx<Vec<VmmProcessMapThreadEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetThread)(self.vmm.native.h, self.pid, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetThread: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_THREAD_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetThread: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapThreadEntry {
                    pid : self.pid,
                    thread_id : ne.dwTID,
                    thread_pid : ne.dwPID,
                    exit_status : ne.dwExitStatus,
                    state : ne.bState,
                    running : ne.bRunning,
                    priority : ne.bPriority,
                    priority_base : ne.bBasePriority,
                    va_ethread : ne.vaETHREAD,
                    va_teb : ne.vaTeb,
                    ft_create_time : ne.ftCreateTime,
                    ft_exit_time : ne.ftExitTime,
                    va_start_address : ne.vaStartAddress,
                    va_win32_start_address : ne.vaWin32StartAddress,
                    va_stack_user_base : ne.vaStackBaseUser,
                    va_stack_user_limit : ne.vaStackLimitUser,
                    va_stack_kernel_base : ne.vaStackBaseKernel,
                    va_stack_kernel_limit : ne.vaStackLimitKernel,
                    va_trap_frame : ne.vaTrapFrame,
                    va_rip : ne.vaRIP,
                    va_rsp : ne.vaRSP,
                    affinity : ne.qwAffinity,
                    user_time : ne.dwUserTime,
                    kernel_time : ne.dwKernelTime,
                    suspend_count : ne.bSuspendCount,
                    wait_reason : ne.bWaitReason
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_unloaded_module(&self) -> ResultEx<Vec<VmmProcessMapUnloadedModuleEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetUnloadedModuleU)(self.vmm.native.h, self.pid, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetUnloadedModuleU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_UNLOADEDMODULE_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetUnloadedModuleU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapUnloadedModuleEntry {
                    pid : self.pid,
                    va_base : ne.vaBase,
                    image_size : ne.cbImageSize,
                    is_wow64 : ne.fWoW64,
                    name : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                    checksum : ne.dwCheckSum,
                    timedatestamp : ne.dwTimeDateStamp,
                    ft_unload : ne.ftUnload,
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_vad(&self, is_identify_modules : bool) -> ResultEx<Vec<VmmProcessMapVadEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetVadU)(self.vmm.native.h, self.pid, is_identify_modules, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetVadU: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_VAD_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetVadU: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapVadEntry {
                    pid : self.pid,
                    va_start : ne.vaStart,
                    va_end : ne.vaEnd,
                    va_vad : ne.vaVad,
                    u0 : ne.u0,
                    u1 : ne.u1,
                    u2 : ne.u2,
                    commit_charge : ne.u1 & 0x7fffffff,
                    is_mem_commit : (ne.u1 & 0x80000000) != 0,
                    cb_prototype_pte : ne.cbPrototypePte,
                    va_prototype_pte : ne.vaPrototypePte,
                    va_subsection : ne.vaSubsection,
                    va_file_object : ne.vaFileObject,
                    info : String::from(CStr::from_ptr(ne.uszText).to_str().unwrap_or("")),
                    vadex_page_base : ne.cVadExPagesBase,
                    vadex_page_count : ne.cVadExPages,
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_vadex(&self, offset_pages : u32, count_pages : u32) -> ResultEx<Vec<VmmProcessMapVadExEntry>> {
        unsafe {
            let mut structs = std::ptr::null_mut();
            let r = (self.vmm.native.VMMDLL_Map_GetVadEx)(self.vmm.native.h, self.pid, offset_pages, count_pages, &mut structs);
            if !r {
                return Err("VMMDLL_Map_GetVadEx: fail.".into());
            }
            if (*structs).dwVersion != VMMDLL_MAP_VADEX_VERSION {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Err("VMMDLL_Map_GetVadEx: bad version.".into());
            }
            let mut result = Vec::new();
            if (*structs).cMap == 0 {
                (self.vmm.native.VMMDLL_MemFree)(structs as usize);
                return Ok(result);
            }
            let cMap : usize = (*structs).cMap.try_into()?;
            let pMap = std::slice::from_raw_parts(&(*structs).pMap, cMap);
            for i in 0..cMap {
                let ne = &pMap[i];
                let e = VmmProcessMapVadExEntry {
                    pid : self.pid,
                    tp : VmmProcessMapVadExType::from(ne.tp),
                    i_pml : ne.iPML,
                    va : ne.va,
                    pa : ne.pa,
                    pte : ne.pte,
                    proto_tp : VmmProcessMapVadExType::from(ne.proto_tp),
                    proto_pa : ne.proto_pa,
                    proto_pte : ne.proto_va,
                    va_vad_base : ne.vaVadBase,
                };
                result.push(e);
            }
            (self.vmm.native.VMMDLL_MemFree)(structs as usize);
            return Ok(result);
        }
    }

    fn impl_map_module_data_directory(&self, module_name : &str) -> ResultEx<Vec<VmmProcessMapDirectoryEntry>> {
        let sz_module_name = CString::new(module_name)?;
        let mut data_directories = vec![CIMAGE_DATA_DIRECTORY::default(); 16];
        let r = (self.vmm.native.VMMDLL_ProcessGetDirectoriesU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr(), data_directories.as_mut_ptr());
        if !r {
            return Err("VMMDLL_ProcessGetDirectoriesU: fail.".into());
        }
        let mut result = Vec::new();
        for i in 0..16 {
            let src : &CIMAGE_DATA_DIRECTORY = data_directories.get(i).unwrap();
            let dst = VmmProcessMapDirectoryEntry {
                pid : self.pid,
                name : DIRECTORY_NAMES[i],
                virtual_address : src.VirtualAddress,
                size : src.Size,
            };
            result.push(dst);
        }
        return Ok(result);
    }

    fn impl_map_module_section(&self, module_name : &str) -> ResultEx<Vec<VmmProcessSectionEntry>> {
        let sz_module_name = CString::new(module_name)?;
        let mut section_count = 0u32;
        let r = (self.vmm.native.VMMDLL_ProcessGetSectionsU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr(), std::ptr::null_mut(), 0, &mut section_count);
        if !r {
            return Err("VMMDLL_ProcessGetSectionsU: fail.".into());
        }
        let mut sections = vec![CIMAGE_SECTION_HEADER::default(); section_count.try_into()?];
        let mut result = Vec::new();
        if section_count == 0 {
            return Ok(result);
        }
        let r = (self.vmm.native.VMMDLL_ProcessGetSectionsU)(self.vmm.native.h, self.pid, sz_module_name.as_ptr(), sections.as_mut_ptr(), section_count, &mut section_count);
        if !r {
            return Err("VMMDLL_ProcessGetSectionsU: fail.".into());
        }
        for i in 0..(section_count as usize) {
            let src : &CIMAGE_SECTION_HEADER = sections.get(i).unwrap();
            let dst = VmmProcessSectionEntry {
                pid : self.pid,
                index : i as u32,
                name : std::str::from_utf8(&src.Name).unwrap_or_default().to_string(),
                name_raw : src.Name,
                misc_virtual_size : src.Misc_VirtualAddress,
                virtual_address : src.VirtualAddress,
                size_of_raw_data : src.SizeOfRawData,
                pointer_to_raw_data : src.PointerToRawData,
                pointer_to_relocations : src.PointerToRelocations,
                pointer_to_linenumbers : src.PointerToLinenumbers,
                number_of_relocations : src.NumberOfRelocations,
                number_of_linenumbers : src.NumberOfLinenumbers,
                characteristics : src.Characteristics,
            };
            result.push(dst);
        }
        return Ok(result);
    }

}






//=============================================================================
// INTERNAL: VMM.SCATTERMEMORY:
//=============================================================================

impl fmt::Display for VmmScatterMemory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.pid == u32::MAX { write!(f, "VmmScatterMemory:physical") } else { write!(f, "VmmScatterMemory:virtual:{}", self.pid) }
    }
}

impl Drop for VmmScatterMemory<'_> {
    fn drop(&mut self) {
        if self.is_scatter_ex {
            let _r = self.impl_execute();
        }
        (self.vmm.native.VMMDLL_Scatter_CloseHandle)(self.hs);
    }
}

impl <'a> VmmScatterMemory<'a> {
    fn impl_prepare_ex(&mut self, data_to_read : &'a mut (u64, Vec<u8>, u32)) -> ResultEx<()> {
        if data_to_read.2 != 0 {
            return Err("data_to_read.2 not set to zero".into());
        }
        let cb = u32::try_from(data_to_read.1.len())?;
        let r = (self.vmm.native.VMMDLL_Scatter_PrepareEx)(self.hs, data_to_read.0, cb, data_to_read.1.as_mut_ptr(), &mut data_to_read.2);
        if !r {
            return Err("VMMDLL_Scatter_PrepareEx: fail.".into());
        }
        self.is_scatter_ex = true;
        return Ok(());
    }

    fn impl_prepare_ex_as<T>(&mut self, data_to_read : &'a mut (u64, T, u32)) -> ResultEx<()> {
        if data_to_read.2 != 0 {
            return Err("data_to_read.2 not set to zero".into());
        }
        let cb = u32::try_from(std::mem::size_of::<T>())?;
        let r = (self.vmm.native.VMMDLL_Scatter_PrepareEx)(self.hs, data_to_read.0, cb, &mut data_to_read.1 as *mut _ as *mut u8, &mut data_to_read.2);
        if !r {
            return Err("VMMDLL_Scatter_PrepareEx: fail.".into());
        }
        self.is_scatter_ex = true;
        return Ok(());
    }
}

impl VmmScatterMemory<'_> {
    fn impl_prepare(&self, va : u64, size : usize) -> ResultEx<()> {
        let cb = u32::try_from(size)?;
        let r = (self.vmm.native.VMMDLL_Scatter_Prepare)(self.hs, va, cb);
        if !r {
            return Err("VMMDLL_Scatter_Prepare: fail.".into());
        }
        return Ok(());
    }

    fn impl_prepare_write(&self, va : u64, data : &Vec<u8>) -> ResultEx<()> {
        let cb = u32::try_from(data.len())?;
        let pb = data.as_ptr();
        let r = (self.vmm.native.VMMDLL_Scatter_PrepareWrite)(self.hs, va, pb, cb);
        if !r {
            return Err("VMMDLL_Scatter_PrepareWrite: fail.".into());
        }
        return Ok(());
    }

    fn impl_prepare_write_as<T>(&self, va : u64, data : &T) -> ResultEx<()> {
        let cb = u32::try_from(std::mem::size_of::<T>())?;
        let r = (self.vmm.native.VMMDLL_Scatter_PrepareWrite)(self.hs, va, data as *const _ as *const u8, cb);
        if !r {
            return Err("VMMDLL_Scatter_PrepareWrite: fail.".into());
        }
        return Ok(());
    }

    fn impl_execute(&self) -> ResultEx<()> {
        let r = (self.vmm.native.VMMDLL_Scatter_Execute)(self.hs);
        if !r {
            return Err("VMMDLL_Scatter_Execute: fail.".into());
        }
        return Ok(());
    }

    fn impl_read(&self, va : u64, size : usize) -> ResultEx<Vec<u8>> {
        let cb = u32::try_from(size)?;
        let mut cb_read = 0;
        let mut pb_result = vec![0u8; size];
        let r = (self.vmm.native.VMMDLL_Scatter_Read)(self.hs, va, cb, pb_result.as_mut_ptr(), &mut cb_read);
        if !r {
            return Err("VMMDLL_Scatter_Read: fail.".into());
        }
        return Ok(pb_result);
    }

    fn impl_read_as<T>(&self, va : u64) -> ResultEx<T> {
        unsafe {
            let cb = u32::try_from(std::mem::size_of::<T>())?;
            let mut cb_read = 0;
            let mut result : T = std::mem::zeroed();
            let r = (self.vmm.native.VMMDLL_Scatter_Read)(self.hs, va, cb, &mut result as *mut _ as *mut u8, &mut cb_read);
            if !r {
                return Err("VMMDLL_Scatter_Read: fail.".into());
            }
            return Ok(result);
        }
    }

    fn impl_clear(&self) -> ResultEx<()> {
        let r = (self.vmm.native.VMMDLL_Scatter_Clear)(self.hs, self.pid, self.flags);
        if !r {
            return Err("VMMDLL_Scatter_Clear: fail.".into());
        }
        return Ok(());
    }
}






//=============================================================================
// INTERNAL: VMM.SEARCH:
//=============================================================================

impl fmt::Display for VmmSearch<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmSearch")
    }
}

impl fmt::Display for VmmSearchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmSearchResult")
    }
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Debug, Default)]
struct CVMMDLL_MEM_SEARCH_CONTEXT_SEARCHENTRY {
    cbAlign : u32,
    cb : u32,
    pb : [u8; 32],
    pbSkipMask : [u8; 32],
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Debug, Default)]
pub(crate) struct CVMMDLL_MEM_SEARCH_CONTEXT {
    dwVersion : u32,
    _Filler : [u32; 2],
    fAbortRequested : u32,
    cMaxResult : u32,
    cSearch : u32,
    search : [CVMMDLL_MEM_SEARCH_CONTEXT_SEARCHENTRY; 16],
    vaMin : u64,
    vaMax : u64,
    vaCurrent : u64,
    _Filler2 : u32,
    cResult : u32,
    cbReadTotal : u64,
    pvUserPtrOpt : usize,
    pfnResultOptCB : usize,
    ReadFlags : u64,
    fForcePTE : u32,
    fForceVAD : u32,
    pfnFilterOptCB : usize,
}

impl Drop for VmmSearch<'_> {
    fn drop(&mut self) {
        if self.is_started && !self.is_completed {
            self.impl_abort();
            let _r = self.impl_result();
        }
    }
}

// The below implementation is quite ugly, but it works since all methods are
// serialized since they all require &mut self. Under no conditions should the
// VmmSearch struct be accessed directly or non-mutable.
impl VmmSearch<'_> {
    fn impl_result(&mut self) -> VmmSearchResult {
        if self.is_started == false {
            self.impl_start();
        }
        if self.is_completed == false {
            self.is_completed = true;
            if let Some(thread) = self.thread.take() {
                if let Ok(thread_result) = thread.join() {
                    self.is_completed_success = thread_result;
                }
            }
        }
        return self.impl_poll();
    }

    fn impl_abort(&mut self) {
        if self.is_started && !self.is_completed {
            self.native_search.fAbortRequested = 1;
        }
    }

    fn impl_start(&mut self) {
        if self.is_started == false {
            self.is_started = true;
            // ugly code below - but it works ...
            self.native_search.pvUserPtrOpt = std::ptr::addr_of!(self.result) as usize;
            let pid = self.pid;
            let native_h = self.vmm.native.h;
            let pfn = self.vmm.native.VMMDLL_MemSearch;
            let ptr = &mut self.native_search as *mut CVMMDLL_MEM_SEARCH_CONTEXT;
            let ptr_wrap = ptr as usize;
            let thread_handle = std::thread::spawn(move || {
                let ptr = ptr_wrap as *mut CVMMDLL_MEM_SEARCH_CONTEXT;
                (pfn)(native_h, pid, ptr, std::ptr::null_mut(), std::ptr::null_mut())
            });
            self.thread = Some(thread_handle);
        }
    }

    fn impl_poll(&mut self) -> VmmSearchResult {
        if self.is_started && !self.is_completed && self.thread.as_ref().unwrap().is_finished() {
            return self.impl_result();
        }
        let result_vec = if self.is_completed_success { self.result.clone() } else { Vec::new() };
        return VmmSearchResult {
            is_started : self.is_started,
            is_completed : self.is_completed,
            is_completed_success : self.is_completed_success,
            addr_min : self.native_search.vaMin,
            addr_max : self.native_search.vaMax,
            addr_current : self.native_search.vaCurrent,
            total_read_bytes : self.native_search.cbReadTotal,
            total_results : self.native_search.cResult,
            result : result_vec,
        }
    }

    fn impl_new<'a>(vmm : &'a Vmm<'a>, pid : u32, addr_min : u64, addr_max : u64, num_results_max : u32, flags : u64) -> ResultEx<VmmSearch<'a>> {
        let num_results_max = std::cmp::min(0x10000, num_results_max);
        let addr_min = addr_min & 0xfffffffffffff000;
        let addr_max = addr_max & 0xfffffffffffff000;
        if addr_max != 0 && addr_max <= addr_min {
            return Err("search max address must be larger than min address".into());
        }
        let result_vec = Vec::new();
        let mut native = CVMMDLL_MEM_SEARCH_CONTEXT::default();
        native.dwVersion = VMMDLL_MEM_SEARCH_VERSION;
        native.vaMin = addr_min;
        native.vaMax = addr_max;
        native.ReadFlags = flags;
        native.cMaxResult = num_results_max;
        native.pfnResultOptCB = VmmSearch::impl_search_cb as usize;
        native.pvUserPtrOpt = std::ptr::addr_of!(result_vec) as usize;
        //let ptr = result_vec::as_mut_ptr;
        return Ok(VmmSearch {
            vmm,
            pid,
            is_started : false,
            is_completed : false,
            is_completed_success : false,
            native_search : native,
            thread : None,
            result : result_vec,
        });
    }

    fn impl_add_search(&mut self, search_bytes : &[u8], search_skipmask : Option<&[u8]>, byte_align : u32) -> ResultEx<u32> {
        if self.native_search.cSearch as usize >= self.native_search.search.len() {
            return Err("Search max terms reached.".into());
        }
        if (search_bytes.len() == 0) || (search_bytes.len() > 32) {
            return Err("Search invalid length: search_bytes.".into());
        }
        if byte_align > 0 {
            if ((byte_align & (byte_align - 1)) != 0) || (byte_align > 0x1000) {
                return Err("Search bad byte_align.".into());
            }
        }
        if let Some(search_skipmask) = search_skipmask {
            if search_skipmask.len() > search_bytes.len() {
                return Err("Search invalid length: search_skipmask.".into());
            }
        }
        let term = &mut self.native_search.search[self.native_search.cSearch as usize];
        term.cbAlign = byte_align;
        term.cb = search_bytes.len() as u32;
        term.pb[0..search_bytes.len()].copy_from_slice(search_bytes);
        if let Some(search_skipmask) = search_skipmask {
            term.pbSkipMask[0..search_skipmask.len()].copy_from_slice(search_skipmask);
        }
        let result_index = self.native_search.cSearch;
        self.native_search.cSearch += 1;
        return Ok(result_index);
    }

    extern "C" fn impl_search_cb(ctx : usize, va : u64, i_search : u32) -> bool {
        unsafe {
            let ctx = ctx as *const CVMMDLL_MEM_SEARCH_CONTEXT;
            let ptr_result_vec = (*ctx).pvUserPtrOpt as *mut Vec<(u64, u32)>;
            (*ptr_result_vec).push((va, i_search));
            return true;
        }
    }
}






//=============================================================================
// INTERNAL: VMM.PLUGINS:
//=============================================================================

impl<T> fmt::Display for VmmPluginContext<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmPluginContext")
    }
}

impl fmt::Display for VmmPluginFileList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmPluginFileList")
    }
}

impl<T> fmt::Display for VmmPluginInitializationContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmPluginInitializationContext")
    }
}

impl fmt::Display for VmmPluginInitializationInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VmmPluginInitializationInfo")
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVMMDLL_PLUGIN_CONTEXT<'a, T> {
    magic : u64,
    wVersion : u16,
    wSize : u16,
    pid : u32,
    pProcess : usize,
    uszModule : *const c_char,
    uszPath : *const c_char,
    pvReserved1 : usize,
    ctxM : *const VmmPluginContext<'a, T>,
    MID : u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CVMMDLL_PLUGIN_REGINFO<T> {
    magic : u64,
    wVersion : u16,
    wSize : u16,
    tpMemoryModel : u32,
    tpSystem : u32,
    hDLL : usize,
    pfnPluginManager_Register : extern "C" fn(H : usize, pPluginRegInfo : *mut CVMMDLL_PLUGIN_REGINFO<T>) -> bool,
    uszPathVmmDLL : *const c_char,
    _Reserved : [u32; 30],
    Py_fPythonStandalone : bool,
    Py__Reserved : u32,
    Py_hReservedDllPython3 : usize,
    Py_hReservedDllPython3X : usize,
    // reg_info:
    reg_info_ctxM : usize,
    reg_info_uszPathName : [u8; 128],
    reg_info_fRootModule : u32,          // bool
    reg_info_fProcessModule : u32,       // bool
    reg_info_fRootModuleHidden : u32,    // bool
    reg_info_fProcessModuleHidden : u32, // bool
    reg_info_sTimelineNameShort : [u8; 6],
    reg_info__Reserved : [u8; 2],
    reg_info_uszTimelineFile : [u8; 32],
    reg_info__Reserved2 : [u8; 32],
    // reg_fn:
    reg_fn_pfnList : extern "C" fn(H : usize, ctxP : *const CVMMDLL_PLUGIN_CONTEXT<T>, pFileList : usize) -> bool,
    reg_fn_pfnRead : extern "C" fn(H : usize, ctxP : *const CVMMDLL_PLUGIN_CONTEXT<T>, pb : *mut u8, cb : u32, pcbRead : *mut u32, cbOffset : u64) -> u32,
    reg_fn_pfnWrite : extern "C" fn(H : usize, ctxP : *const CVMMDLL_PLUGIN_CONTEXT<T>, pb : *const u8, cb : u32, pcbWrite : *mut u32, cbOffset : u64) -> u32,
    reg_fn_pfnNotify : extern "C" fn(H : usize, ctxP : *const CVMMDLL_PLUGIN_CONTEXT<T>, fEvent : u32, pvEvent : usize, cbEvent : usize),
    reg_fn_pfnClose : extern "C" fn(H : usize, ctxP : *const CVMMDLL_PLUGIN_CONTEXT<T>),
    reg_fn_pfnVisibleModule : extern "C" fn(H : usize, ctxP : *const CVMMDLL_PLUGIN_CONTEXT<T>) -> bool,
    reg_fn_pvReserved : [usize; 10],
    // reg_fnfc: // TODO:
    reg_fnfc_pfnInitialize : usize,
    reg_fnfc_pfnFinalize : usize,
    reg_fnfc_pfnTimeline : usize,
    reg_fnfc_pfnIngestPhysmem : usize,
    reg_fnfc_pfnIngestVirtmem : usize,
    reg_fnfc_pfnIngestFinalize : usize,
    reg_fnfc_pvReserved : [usize; 8],
    reg_fnfc_pfnLogCSV : usize,
    reg_fnfc_pfnLogJSON : usize,
    // sysinfo:
    sysinfo_f32 : u32,
    sysinfo_dwVersionMajor : u32,
    sysinfo_dwVersionMinor : u32,
    sysinfo_dwVersionBuild : u32,
    sysinfo__Reserved : [u32; 32],
}

fn impl_new_plugin_initialization<T>(native_h : usize, native_reginfo : usize) -> ResultEx<(VmmPluginInitializationInfo, VmmPluginInitializationContext<T>)> {
    unsafe {
        let reginfo = native_reginfo as *mut CVMMDLL_PLUGIN_REGINFO<T>;
        if (*reginfo).magic != VMMDLL_PLUGIN_REGINFO_MAGIC || (*reginfo).wVersion != VMMDLL_PLUGIN_REGINFO_VERSION {
            return Err("Bad reginfo magic/version.".into());
        }
        let info = VmmPluginInitializationInfo {
            tp_system : VmmSystemType::from((*reginfo).tpSystem),
            tp_memorymodel : VmmMemoryModelType::from((*reginfo).tpMemoryModel),
            version_major : (*reginfo).sysinfo_dwVersionMajor,
            version_minor : (*reginfo).sysinfo_dwVersionMinor,
            version_build : (*reginfo).sysinfo_dwVersionBuild,
        };
        let ctx = VmmPluginInitializationContext {
            h_vmm : native_h,
            h_reginfo : native_reginfo,
            ctx : None,
            path_name : String::from(""),
            is_root_module : false,
            is_root_module_hidden : false,
            is_process_module : false,
            is_process_module_hidden : false,
            fn_list : None,
            fn_read : None,
            fn_write : None,
            fn_notify : None,
            fn_visible : None,
        };
        return Ok((info, ctx));
    }
}

impl<T> VmmPluginInitializationContext<T> {
    fn impl_register(self) -> ResultEx<()> {
        unsafe {
            let mut reginfo = self.h_reginfo as *mut CVMMDLL_PLUGIN_REGINFO<T>;
            if (*reginfo).magic != VMMDLL_PLUGIN_REGINFO_MAGIC || (*reginfo).wVersion != VMMDLL_PLUGIN_REGINFO_VERSION {
                return Err("Bad reginfo magic/version.".into());
            }
            if self.ctx.is_none() {
                return Err("User context ctx is missing. User context cannot be None.".into());
            }
            let pathname_str = str::replace(&self.path_name, "/", "\\");
            let pathname_cstring = CString::new(pathname_str)?;
            let pathname_bytes = pathname_cstring.to_bytes_with_nul();
            if pathname_bytes.len() > (*reginfo).reg_info_uszPathName.len() {
                return Err("Plugin path/name too long.".into());
            }
            let pathname_len = std::cmp::min(pathname_bytes.len(), (*reginfo).reg_info_uszPathName.len());
            // "initialize" rust vmm context from handle and create rust plugin native context:
            let c_path_vmm = CStr::from_ptr((*reginfo).uszPathVmmDLL);
            let vmm = impl_new(c_path_vmm.to_str()?, self.h_vmm, &Vec::new())?;
            let ctx_user = self.ctx.unwrap();
            let ctx_rust = VmmPluginContext {
                vmm : vmm,
                ctxlock : std::sync::RwLock::new(ctx_user),
                fn_list : self.fn_list,
                fn_read : self.fn_read,
                fn_write : self.fn_write,
                fn_notify : self.fn_notify,
                fn_visible : self.fn_visible,
            };
            let ctx_rust_box = Box::new(ctx_rust);
            let ctx_native = Box::into_raw(ctx_rust_box);
            // prepare native registration context and register:
            for i in 0..pathname_len {
                (*reginfo).reg_info_uszPathName[i] = pathname_bytes[i];
            }
            (*reginfo).reg_info_ctxM = ctx_native as usize;
            (*reginfo).reg_info_fProcessModule = if self.is_process_module { 1 }  else { 0 };
            (*reginfo).reg_info_fProcessModuleHidden = if self.is_process_module_hidden { 1 }  else { 0 };
            (*reginfo).reg_info_fRootModule = if self.is_root_module { 1 }  else { 0 };
            (*reginfo).reg_info_fRootModuleHidden = if self.is_root_module_hidden { 1 }  else { 0 };
            // native callback registration:
            (*reginfo).reg_fn_pfnClose = impl_plugin_close_cb;
            if self.fn_list.is_some() {
                (*reginfo).reg_fn_pfnList = impl_plugin_list_cb;
            }
            if self.fn_read.is_some() {
                (*reginfo).reg_fn_pfnRead = impl_plugin_read_cb;
            }
            if self.fn_write.is_some() {
                (*reginfo).reg_fn_pfnWrite = impl_plugin_write_cb;
            }
            if self.fn_visible.is_some() {
                (*reginfo).reg_fn_pfnVisibleModule = impl_plugin_visible_cb;
            }
            if self.fn_notify.is_some() {
                (*reginfo).reg_fn_pfnNotify = impl_plugin_notify_cb;
            }
            let r = ((*reginfo).pfnPluginManager_Register)(self.h_vmm, reginfo);
            if !r {
                return Err("Failed registering plugin.".into());
            }
            return Ok(());
        }
    }
}

impl VmmPluginFileList<'_> {
    fn impl_add_file(&self, name : &str, size : u64) {
        let sz_name = CString::new(name).unwrap();
        (self.vmm.native.VMMDLL_VfsList_AddFile)(self.h_file_list, sz_name.as_ptr(), size, 0);
    }

    fn impl_add_directory(&self, name : &str) {
        let sz_name = CString::new(name).unwrap();
        (self.vmm.native.VMMDLL_VfsList_AddDirectory)(self.h_file_list, sz_name.as_ptr(), 0);
    }
}

extern "C" fn impl_plugin_close_cb<T>(_h : usize, ctxp : *const CVMMDLL_PLUGIN_CONTEXT<T>) {
    unsafe {
        drop(Box::from_raw((*ctxp).ctxM as *mut VmmPluginContext<T>));
    }
    println!("RUST: PLUGIN CLOSE");
}

extern "C" fn impl_plugin_list_cb<T>(_h : usize, ctxp : *const CVMMDLL_PLUGIN_CONTEXT<T>, h_pfilelist : usize) -> bool {
    unsafe {
        let ctx = &*(*ctxp).ctxM;
        if ((*ctxp).magic != VMMDLL_PLUGIN_CONTEXT_MAGIC) || ((*ctxp).wVersion != VMMDLL_PLUGIN_CONTEXT_VERSION) {
            return true;
        }
        let callback = ctx.fn_list.unwrap();
        let process = if (*ctxp).pid > 0 { Some(VmmProcess{ vmm : &ctx.vmm, pid : (*ctxp).pid }) } else { None };
        let path_string = str::replace(CStr::from_ptr((*ctxp).uszPath).to_str().unwrap_or("[err]"), "\\", "/");
        let path = path_string.as_str();
        if path == "[err]" {
            return true;
        }
        let filelist = VmmPluginFileList {
            vmm : &ctx.vmm,
            h_file_list : h_pfilelist,
        };
        let _r = (callback)(ctx, process, path, &filelist);
        return true;
    }
}

extern "C" fn impl_plugin_read_cb<T>(_h : usize, ctxp : *const CVMMDLL_PLUGIN_CONTEXT<T>, pb : *mut u8, cb : u32, pcb_read : *mut u32, cb_offset : u64) -> u32 {
    unsafe {
        *pcb_read = 0;
        let ctx = &*(*ctxp).ctxM;
        if ((*ctxp).magic != VMMDLL_PLUGIN_CONTEXT_MAGIC) || ((*ctxp).wVersion != VMMDLL_PLUGIN_CONTEXT_VERSION) {
            return VMMDLL_STATUS_FILE_INVALID;
        }
        let callback = ctx.fn_read.unwrap();
        let process = if (*ctxp).pid > 0 { Some(VmmProcess{ vmm : &ctx.vmm, pid : (*ctxp).pid }) } else { None };
        let path_string = str::replace(CStr::from_ptr((*ctxp).uszPath).to_str().unwrap_or("[err]"), "\\", "/");
        let path = path_string.as_str();
        if path == "[err]" {
            return VMMDLL_STATUS_FILE_INVALID;
        }
        let r = match (callback)(ctx, process, path, cb, cb_offset) {
            Err(_) => return VMMDLL_STATUS_FILE_INVALID,
            Ok(r) => r,
        };
        if r.len() == 0 {
            return VMMDLL_STATUS_END_OF_FILE;
        }
        if r.len() > u32::MAX as usize {
            return VMMDLL_STATUS_FILE_INVALID;
        }
        *pcb_read = r.len() as u32;
        std::ptr::copy_nonoverlapping(r.as_ptr(), pb, r.len());
        return VMMDLL_STATUS_SUCCESS;
    }
}

extern "C" fn impl_plugin_write_cb<T>(_h : usize, ctxp : *const CVMMDLL_PLUGIN_CONTEXT<T>, pb : *const u8, cb : u32, pcb_write : *mut u32, cb_offset : u64) -> u32 {
    unsafe {
        *pcb_write = 0;
        let ctx = &*(*ctxp).ctxM;
        if ((*ctxp).magic != VMMDLL_PLUGIN_CONTEXT_MAGIC) || ((*ctxp).wVersion != VMMDLL_PLUGIN_CONTEXT_VERSION) {
            return VMMDLL_STATUS_FILE_INVALID;
        }
        let callback = ctx.fn_write.unwrap();
        let process = if (*ctxp).pid > 0 { Some(VmmProcess{ vmm : &ctx.vmm, pid : (*ctxp).pid }) } else { None };
        let path_string = str::replace(CStr::from_ptr((*ctxp).uszPath).to_str().unwrap_or("[err]"), "\\", "/");
        let path = path_string.as_str();
        if path == "[err]" {
            return VMMDLL_STATUS_FILE_INVALID;
        }
        let size = cb as usize;
        let mut data = vec![0u8; size];
        std::ptr::copy_nonoverlapping(pb, data.as_mut_ptr(), size);
        if (callback)(ctx, process, path, data, cb_offset).is_err() {
            return VMMDLL_STATUS_FILE_INVALID;
        };
        *pcb_write = cb;
        return VMMDLL_STATUS_SUCCESS;
    }
}

extern "C" fn impl_plugin_visible_cb<T>(_h : usize, ctxp : *const CVMMDLL_PLUGIN_CONTEXT<T>) -> bool {
    unsafe {
        let ctx = &*(*ctxp).ctxM;
        if ((*ctxp).magic != VMMDLL_PLUGIN_CONTEXT_MAGIC) || ((*ctxp).wVersion != VMMDLL_PLUGIN_CONTEXT_VERSION) {
            return false;
        }
        let callback = ctx.fn_visible.unwrap();
        let process = if (*ctxp).pid > 0 { Some(VmmProcess{ vmm : &ctx.vmm, pid : (*ctxp).pid }) } else { None };
        let path_string = str::replace(CStr::from_ptr((*ctxp).uszPath).to_str().unwrap_or("[err]"), "\\", "/");
        let path = path_string.as_str();
        if path == "[err]" {
            return false;
        }
        return (callback)(ctx, process).unwrap_or(false);
    }
}

extern "C" fn impl_plugin_notify_cb<T>(_h : usize, ctxp : *const CVMMDLL_PLUGIN_CONTEXT<T>, f_event : u32, _pv_event : usize, _cb_event : usize) {
    unsafe {
        let ctx = &*(*ctxp).ctxM;
        if ((*ctxp).magic != VMMDLL_PLUGIN_CONTEXT_MAGIC) || ((*ctxp).wVersion != VMMDLL_PLUGIN_CONTEXT_VERSION) {
            return;
        }
        let callback = ctx.fn_notify.unwrap();
        let _r = (callback)(ctx, f_event);
    }
}
