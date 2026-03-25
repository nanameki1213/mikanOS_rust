use core::ffi::c_void;

// ──────────────────────────────────────────────────────────────────────────────
// Primitive type aliases
// ──────────────────────────────────────────────────────────────────────────────

/// EFI_STATUS — UEFI ステータスコード一覧 (UEFI Spec 2.10 Appendix D)
///
/// エラーコードは最上位ビット (bit 63) が立っている。
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    // ── 成功 ──────────────────────────────────────────────────────────────────
    Success = 0,

    // ── 警告 (bit63 = 0) ──────────────────────────────────────────────────────
    WarnUnknownGlyph = 1,
    WarnDeleteFailure = 2,
    WarnWriteFailure = 3,
    WarnBufferTooSmall = 4,
    WarnStaleData = 5,
    WarnFileSystem = 6,
    WarnResetRequired = 7,

    // ── エラー (bit63 = 1) ────────────────────────────────────────────────────
    LoadError = 0x8000_0000_0000_0001,
    InvalidParameter = 0x8000_0000_0000_0002,
    Unsupported = 0x8000_0000_0000_0003,
    BadBufferSize = 0x8000_0000_0000_0004,
    BufferTooSmall = 0x8000_0000_0000_0005,
    NotReady = 0x8000_0000_0000_0006,
    DeviceError = 0x8000_0000_0000_0007,
    WriteProtected = 0x8000_0000_0000_0008,
    OutOfResources = 0x8000_0000_0000_0009,
    VolumeCorrupted = 0x8000_0000_0000_000A,
    VolumeFull = 0x8000_0000_0000_000B,
    NoMedia = 0x8000_0000_0000_000C,
    MediaChanged = 0x8000_0000_0000_000D,
    NotFound = 0x8000_0000_0000_000E,
    AccessDenied = 0x8000_0000_0000_000F,
    NoResponse = 0x8000_0000_0000_0010,
    NoMapping = 0x8000_0000_0000_0011,
    Timeout = 0x8000_0000_0000_0012,
    NotStarted = 0x8000_0000_0000_0013,
    AlreadyStarted = 0x8000_0000_0000_0014,
    Aborted = 0x8000_0000_0000_0015,
    IcmpError = 0x8000_0000_0000_0016,
    TftpError = 0x8000_0000_0000_0017,
    ProtocolError = 0x8000_0000_0000_0018,
    IncompatibleVersion = 0x8000_0000_0000_0019,
    SecurityViolation = 0x8000_0000_0000_001A,
    CrcError = 0x8000_0000_0000_001B,
    EndOfMedia = 0x8000_0000_0000_001C,
    EndOfFile = 0x8000_0000_0000_001F,
    InvalidLanguage = 0x8000_0000_0000_0020,
    CompromisedData = 0x8000_0000_0000_0021,
    HttpError = 0x8000_0000_0000_0023,
}

impl Status {
    /// エラーコード（bit63 = 1）かどうかを返す
    #[inline]
    pub fn is_error(self) -> bool {
        (self as usize) & (1 << 63) != 0
    }

    /// 警告コード（0 < value < ERROR_BIT）かどうかを返す
    #[inline]
    pub fn is_warning(self) -> bool {
        let v = self as usize;
        v != 0 && v & (1 << 63) == 0
    }
}

/// EFI_EVENT
pub type Event = *mut c_void;

/// EFI_HANDLE
pub type Handle = *mut c_void;

/// EFI_TPL (Task Priority Level)
pub type Tpl = usize;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

// ──────────────────────────────────────────────────────────────────────────────
// EFI_GUID
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_TABLE_HEADER
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct TableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}

// ──────────────────────────────────────────────────────────────────────────────
// Memory types
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub enum AllocateType {
    AllocateAnyPages,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType,
}

#[repr(C)]
pub enum MemoryType {
    ReservedMemoryType,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    AcpiReclaimMemory,
    AcpiMemoryNvs,
    MemoryMappedIo,
    MemoryMappedIoPortSpace,
    PalCode,
    PersistentMemory,
    MaxMemoryType,
}

/// EFI_MEMORY_DESCRIPTOR
#[repr(C)]
pub struct MemoryDescriptor {
    pub memory_type: u32,
    pub physical_start: PhysicalAddress,
    pub virtual_start: VirtualAddress,
    pub number_of_pages: u64,
    pub attribute: u64,
}

// ──────────────────────────────────────────────────────────────────────────────
// Event / Timer types
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub enum TimerDelay {
    TimerCancel,
    TimerPeriodic,
    TimerRelative,
}

pub type EventNotify = unsafe extern "efiapi" fn(event: Event, context: *mut c_void);

// ──────────────────────────────────────────────────────────────────────────────
// Protocol handler types
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub enum InterfaceType {
    NativeInterface,
}

#[repr(C)]
pub enum LocateSearchType {
    AllHandles,
    ByRegisterNotify,
    ByProtocol,
}

#[repr(C)]
pub struct OpenProtocolInformationEntry {
    pub agent_handle: Handle,
    pub controller_handle: Handle,
    pub attributes: u32,
    pub open_count: u32,
}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_DEVICE_PATH_PROTOCOL (opaque header)
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct DevicePath {
    pub r#type: u8,
    pub sub_type: u8,
    pub length: [u8; 2],
}

// ──────────────────────────────────────────────────────────────────────────────
// Time
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub pad1: u8,
    pub nanosecond: u32,
    pub time_zone: i16,
    pub daylight: u8,
    pub pad2: u8,
}

#[repr(C)]
pub struct TimeCapabilities {
    pub resolution: u32,
    pub accuracy: u32,
    pub sets_to_zero: bool,
}

// ──────────────────────────────────────────────────────────────────────────────
// Reset / Capsule types
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub enum ResetType {
    ResetCold,
    ResetWarm,
    ResetShutdown,
    ResetPlatformSpecific,
}

#[repr(C)]
pub struct CapsuleHeader {
    pub capsule_guid: Guid,
    pub header_size: u32,
    pub flags: u32,
    pub capsule_image_size: u32,
}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_CONFIGURATION_TABLE
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct ConfigurationTable {
    pub vendor_guid: Guid,
    pub vendor_table: *mut c_void,
}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_SIMPLE_TEXT_INPUT_PROTOCOL
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct InputKey {
    pub scan_code: u16,
    pub unicode_char: u16,
}

#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verify: bool) -> Status,

    pub read_key_stroke: unsafe extern "efiapi" fn(this: *mut Self, key: *mut InputKey) -> Status,

    pub wait_for_key: Event,
}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct SimpleTextOutputMode {
    pub max_mode: i32,
    pub mode: i32,
    pub attribute: i32,
    pub cursor_column: i32,
    pub cursor_row: i32,
    pub cursor_visible: bool,
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verify: bool) -> Status,

    pub output_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const u16) -> Status,

    pub test_string: unsafe extern "efiapi" fn(this: *mut Self, string: *const u16) -> Status,

    pub query_mode: unsafe extern "efiapi" fn(
        this: *mut Self,
        mode_number: usize,
        columns: *mut usize,
        rows: *mut usize,
    ) -> Status,

    pub set_mode: unsafe extern "efiapi" fn(this: *mut Self, mode_number: usize) -> Status,

    pub set_attribute: unsafe extern "efiapi" fn(this: *mut Self, attribute: usize) -> Status,

    pub clear_screen: unsafe extern "efiapi" fn(this: *mut Self) -> Status,

    pub set_cursor_position:
        unsafe extern "efiapi" fn(this: *mut Self, column: usize, row: usize) -> Status,

    pub enable_cursor: unsafe extern "efiapi" fn(this: *mut Self, visible: bool) -> Status,

    pub mode: *mut SimpleTextOutputMode,
}

// UEFI はシングルスレッド環境であり、実際にはスレッド間共有は発生しない。
// raw pointer フィールドにより Sync が自動実装されないため手動で宣言する。
unsafe impl Sync for SimpleTextOutputProtocol {}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_BOOT_SERVICES
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct BootServices {
    pub header: TableHeader,

    // ── Task Priority Services ────────────────────────────────────────────────
    pub raise_tpl: unsafe extern "efiapi" fn(new_tpl: Tpl) -> Tpl,

    pub restore_tpl: unsafe extern "efiapi" fn(old_tpl: Tpl),

    // ── Memory Services ───────────────────────────────────────────────────────
    pub allocate_pages: unsafe extern "efiapi" fn(
        allocate_type: AllocateType,
        memory_type: MemoryType,
        pages: usize,
        memory: *mut PhysicalAddress,
    ) -> Status,

    pub free_pages: unsafe extern "efiapi" fn(memory: PhysicalAddress, pages: usize) -> Status,

    pub get_memory_map: unsafe extern "efiapi" fn(
        memory_map_size: *mut usize,
        memory_map: *mut MemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> Status,

    pub allocate_pool: unsafe extern "efiapi" fn(
        pool_type: MemoryType,
        size: usize,
        buffer: *mut *mut c_void,
    ) -> Status,

    pub free_pool: unsafe extern "efiapi" fn(buffer: *mut c_void) -> Status,

    // ── Event & Timer Services ────────────────────────────────────────────────
    pub create_event: unsafe extern "efiapi" fn(
        r#type: u32,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotify>,
        context: *mut c_void,
        event: *mut Event,
    ) -> Status,

    pub set_timer:
        unsafe extern "efiapi" fn(event: Event, r#type: TimerDelay, trigger_time: u64) -> Status,

    pub wait_for_event: unsafe extern "efiapi" fn(
        number_of_events: usize,
        event: *mut Event,
        index: *mut usize,
    ) -> Status,

    pub signal_event: unsafe extern "efiapi" fn(event: Event) -> Status,

    pub close_event: unsafe extern "efiapi" fn(event: Event) -> Status,

    pub check_event: unsafe extern "efiapi" fn(event: Event) -> Status,

    // ── Protocol Handler Services ─────────────────────────────────────────────
    pub install_protocol_interface: unsafe extern "efiapi" fn(
        handle: *mut Handle,
        protocol: *mut Guid,
        interface_type: InterfaceType,
        interface: *mut c_void,
    ) -> Status,

    pub reinstall_protocol_interface: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *mut Guid,
        old_interface: *mut c_void,
        new_interface: *mut c_void,
    ) -> Status,

    pub uninstall_protocol_interface: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *mut Guid,
        interface: *mut c_void,
    ) -> Status,

    pub handle_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *mut Guid,
        interface: *mut *mut c_void,
    ) -> Status,

    pub reserved: *mut c_void,

    pub register_protocol_notify: unsafe extern "efiapi" fn(
        protocol: *mut Guid,
        event: Event,
        registration: *mut *mut c_void,
    ) -> Status,

    pub locate_handle: unsafe extern "efiapi" fn(
        search_type: LocateSearchType,
        protocol: *mut Guid,
        search_key: *mut c_void,
        buffer_size: *mut usize,
        buffer: *mut Handle,
    ) -> Status,

    pub locate_device_path: unsafe extern "efiapi" fn(
        protocol: *mut Guid,
        device_path: *mut *mut DevicePath,
        device: *mut Handle,
    ) -> Status,

    pub install_configuration_table:
        unsafe extern "efiapi" fn(guid: *mut Guid, table: *mut c_void) -> Status,

    // ── Image Services ────────────────────────────────────────────────────────
    pub load_image: unsafe extern "efiapi" fn(
        boot_policy: bool,
        parent_image_handle: Handle,
        device_path: *mut DevicePath,
        source_buffer: *mut c_void,
        source_size: usize,
        image_handle: *mut Handle,
    ) -> Status,

    pub start_image: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_data_size: *mut usize,
        exit_data: *mut *mut u16,
    ) -> Status,

    pub exit: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_status: Status,
        exit_data_size: usize,
        exit_data: *mut u16,
    ) -> Status,

    pub unload_image: unsafe extern "efiapi" fn(image_handle: Handle) -> Status,

    pub exit_boot_services:
        unsafe extern "efiapi" fn(image_handle: Handle, map_key: usize) -> Status,

    // ── Miscellaneous Services ────────────────────────────────────────────────
    pub get_next_monotonic_count: unsafe extern "efiapi" fn(count: *mut u64) -> Status,

    pub stall: unsafe extern "efiapi" fn(microseconds: usize) -> Status,

    pub set_watchdog_timer: unsafe extern "efiapi" fn(
        timeout: usize,
        watchdog_code: u64,
        data_size: usize,
        watchdog_data: *mut u16,
    ) -> Status,

    // ── Driver Support Services ───────────────────────────────────────────────
    pub connect_controller: unsafe extern "efiapi" fn(
        controller_handle: Handle,
        driver_image_handle: *mut Handle,
        remaining_device_path: *mut DevicePath,
        recursive: bool,
    ) -> Status,

    pub disconnect_controller: unsafe extern "efiapi" fn(
        controller_handle: Handle,
        driver_image_handle: Handle,
        child_handle: Handle,
    ) -> Status,

    // ── Open and Close Protocol Services ─────────────────────────────────────
    pub open_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *mut Guid,
        interface: *mut *mut c_void,
        agent_handle: Handle,
        controller_handle: Handle,
        attributes: u32,
    ) -> Status,

    pub close_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *mut Guid,
        agent_handle: Handle,
        controller_handle: Handle,
    ) -> Status,

    pub open_protocol_information: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *mut Guid,
        entry_buffer: *mut *mut OpenProtocolInformationEntry,
        entry_count: *mut usize,
    ) -> Status,

    // ── Library Services ──────────────────────────────────────────────────────
    pub protocols_per_handle: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol_buffer: *mut *mut *mut Guid,
        protocol_buffer_count: *mut usize,
    ) -> Status,

    pub locate_handle_buffer: unsafe extern "efiapi" fn(
        search_type: LocateSearchType,
        protocol: *mut Guid,
        search_key: *mut c_void,
        no_handles: *mut usize,
        buffer: *mut *mut Handle,
    ) -> Status,

    pub locate_protocol: unsafe extern "efiapi" fn(
        protocol: *mut Guid,
        registration: *mut c_void,
        interface: *mut *mut c_void,
    ) -> Status,

    /// Variadic — represented as a raw pointer; cast to the appropriate
    /// function type when calling.
    pub install_multiple_protocol_interfaces: *const c_void,

    /// Variadic — represented as a raw pointer; cast to the appropriate
    /// function type when calling.
    pub uninstall_multiple_protocol_interfaces: *const c_void,

    // ── 32-bit CRC Services ───────────────────────────────────────────────────
    pub calculate_crc32:
        unsafe extern "efiapi" fn(data: *mut c_void, data_size: usize, crc32: *mut u32) -> Status,

    // ── Miscellaneous Services ────────────────────────────────────────────────
    pub copy_mem:
        unsafe extern "efiapi" fn(destination: *mut c_void, source: *const c_void, length: usize),

    pub set_mem: unsafe extern "efiapi" fn(buffer: *mut c_void, size: usize, value: u8),

    pub create_event_ex: unsafe extern "efiapi" fn(
        r#type: u32,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotify>,
        context: *const c_void,
        event_group: *const Guid,
        event: *mut Event,
    ) -> Status,
}

// UEFI はシングルスレッド環境であり、実際にはスレッド間共有は発生しない。
// raw pointer フィールドにより Sync が自動実装されないため手動で宣言する。
unsafe impl Sync for BootServices {}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_RUNTIME_SERVICES
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct RuntimeServices {
    pub header: TableHeader,

    // ── Time Services ─────────────────────────────────────────────────────────
    pub get_time:
        unsafe extern "efiapi" fn(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status,

    pub set_time: unsafe extern "efiapi" fn(time: *const Time) -> Status,

    pub get_wakeup_time: unsafe extern "efiapi" fn(
        enabled: *mut bool,
        pending: *mut bool,
        time: *mut Time,
    ) -> Status,

    pub set_wakeup_time: unsafe extern "efiapi" fn(enable: bool, time: *mut Time) -> Status,

    // ── Virtual Memory Services ───────────────────────────────────────────────
    pub set_virtual_address_map: unsafe extern "efiapi" fn(
        memory_map_size: usize,
        descriptor_size: usize,
        descriptor_version: u32,
        virtual_map: *mut MemoryDescriptor,
    ) -> Status,

    pub convert_pointer:
        unsafe extern "efiapi" fn(debug_disposition: usize, address: *mut *mut c_void) -> Status,

    // ── Variable Services ─────────────────────────────────────────────────────
    pub get_variable: unsafe extern "efiapi" fn(
        variable_name: *const u16,
        vendor_guid: *const Guid,
        attributes: *mut u32,
        data_size: *mut usize,
        data: *mut c_void,
    ) -> Status,

    pub get_next_variable_name: unsafe extern "efiapi" fn(
        variable_name_size: *mut usize,
        variable_name: *mut u16,
        vendor_guid: *mut Guid,
    ) -> Status,

    pub set_variable: unsafe extern "efiapi" fn(
        variable_name: *const u16,
        vendor_guid: *const Guid,
        attributes: u32,
        data_size: usize,
        data: *mut c_void,
    ) -> Status,

    // ── Miscellaneous Services ────────────────────────────────────────────────
    pub get_next_high_monotonic_count: unsafe extern "efiapi" fn(high_count: *mut u32) -> Status,

    pub reset_system: unsafe extern "efiapi" fn(
        reset_type: ResetType,
        reset_status: Status,
        data_size: usize,
        reset_data: *mut c_void,
    ),

    // ── UEFI 2.0 Capsule Services ─────────────────────────────────────────────
    pub update_capsule: unsafe extern "efiapi" fn(
        capsule_header_array: *mut *mut CapsuleHeader,
        capsule_count: usize,
        scatter_gather_list: PhysicalAddress,
    ) -> Status,

    pub query_capsule_capabilities: unsafe extern "efiapi" fn(
        capsule_header_array: *mut *mut CapsuleHeader,
        capsule_count: usize,
        maximum_capsule_size: *mut u64,
        reset_type: *mut ResetType,
    ) -> Status,

    // ── Miscellaneous UEFI 2.0 Service ────────────────────────────────────────
    pub query_variable_info: unsafe extern "efiapi" fn(
        attributes: u32,
        maximum_variable_storage_size: *mut u64,
        remaining_variable_storage_size: *mut u64,
        maximum_variable_size: *mut u64,
    ) -> Status,
}

// ──────────────────────────────────────────────────────────────────────────────
// EFI_SYSTEM_TABLE
// ──────────────────────────────────────────────────────────────────────────────

#[repr(C)]
pub struct SystemTable {
    pub header: TableHeader,
    pub firmware_vendor: *const u16,
    pub firmware_revision: u32,
    pub console_in_handle: Handle,
    pub con_in: *mut SimpleTextInputProtocol,
    pub console_out_handle: Handle,
    pub con_out: *mut SimpleTextOutputProtocol,
    pub standard_error_handle: Handle,
    pub std_err: *mut SimpleTextOutputProtocol,
    pub runtime_services: *mut RuntimeServices,
    pub boot_services: *mut BootServices,
    pub number_of_table_entries: usize,
    pub configuration_table: *mut ConfigurationTable,
}
