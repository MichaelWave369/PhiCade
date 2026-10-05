use libloading::Library;
use phicade_runtime::{
    ActionEnvelope, ActionKind, AudioBuffer, CapabilityStatus, CoreError, EmulatorCore,
    FrameBuffer, GameImage, RuntimeCapabilities, RuntimeCapabilityManifest,
    RuntimeExecutionModel, RuntimeQualificationProfile, SystemCommand, SystemId,
};
use serde::Serialize;
use std::{
    ffi::{c_char, c_uint, c_void, CStr, CString},
    fs,
    path::{Path, PathBuf},
    ptr, slice,
    sync::{Mutex, OnceLock},
};

pub const SAMEBOY_SOURCE_REVISION: &str = "213a12ce93d66b105a113debd9396306066a7cfc";
pub const SAMEBOY_VERSION: &str = "1.0.3";
pub const SAMEBOY_LICENSE: &str = "Expat";
pub const LIBRETRO_ADAPTER_ID: &str = "phicade.libretro";
pub const SAMEBOY_QUALIFICATION_PROFILE_ID: &str = "phicade.sameboy-1.0.3-qualified.v1";

const RETRO_DEVICE_JOYPAD: c_uint = 1;
const RETRO_MEMORY_SAVE_RAM: c_uint = 0;
const B: c_uint = 0;
const SELECT: c_uint = 2;
const START: c_uint = 3;
const UP: c_uint = 4;
const DOWN: c_uint = 5;
const LEFT: c_uint = 6;
const RIGHT: c_uint = 7;
const A: c_uint = 8;

const GET_CAN_DUPE: c_uint = 3;
const SHUTDOWN: c_uint = 7;
const GET_SYSTEM_DIRECTORY: c_uint = 9;
const SET_PIXEL_FORMAT: c_uint = 10;
const SET_INPUT_DESCRIPTORS: c_uint = 11;
const GET_VARIABLE: c_uint = 15;
const SET_VARIABLES: c_uint = 16;
const GET_VARIABLE_UPDATE: c_uint = 17;
const GET_SAVE_DIRECTORY: c_uint = 31;
const SET_SYSTEM_AV_INFO: c_uint = 32;
const SET_CONTROLLER_INFO: c_uint = 35;
const SET_GEOMETRY: c_uint = 37;
const GET_INPUT_BITMASKS: c_uint = 51;
const GET_CORE_OPTIONS_VERSION: c_uint = 52;
const SET_CORE_OPTIONS: c_uint = 53;
const SET_CORE_OPTIONS_INTL: c_uint = 54;
const SET_CORE_OPTIONS_DISPLAY: c_uint = 55;
const SET_CORE_OPTIONS_V2: c_uint = 67;
const SET_CORE_OPTIONS_V2_INTL: c_uint = 68;
const SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK: c_uint = 69;

#[derive(Debug, Clone, Copy, Default)]
enum PixelFormat {
    ZeroRgb1555,
    #[default]
    Xrgb8888,
    Rgb565,
}

#[derive(Debug, Clone, Default)]
struct RawVideo {
    width: u32,
    height: u32,
    pitch: usize,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct CallbackState {
    format: PixelFormat,
    video: RawVideo,
    audio: Vec<i16>,
    input_mask: u16,
    system_dir: CString,
    save_dir: CString,
    variables: Vec<(CString, CString)>,
    shutdown: bool,
}

impl Default for CallbackState {
    fn default() -> Self {
        Self {
            format: PixelFormat::Xrgb8888,
            video: RawVideo::default(),
            audio: Vec::new(),
            input_mask: 0,
            system_dir: CString::new(".").unwrap(),
            save_dir: CString::new(".").unwrap(),
            variables: Vec::new(),
            shutdown: false,
        }
    }
}

static CALLBACKS: OnceLock<Mutex<CallbackState>> = OnceLock::new();
fn callbacks() -> &'static Mutex<CallbackState> {
    CALLBACKS.get_or_init(|| Mutex::new(CallbackState::default()))
}

#[repr(C)]
#[derive(Default)]
struct RetroSystemInfo {
    library_name: *const c_char,
    library_version: *const c_char,
    valid_extensions: *const c_char,
    need_fullpath: bool,
    block_extract: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RetroGameGeometry {
    base_width: c_uint,
    base_height: c_uint,
    max_width: c_uint,
    max_height: c_uint,
    aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RetroSystemTiming {
    fps: f64,
    sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RetroSystemAvInfo {
    geometry: RetroGameGeometry,
    timing: RetroSystemTiming,
}

#[repr(C)]
struct RetroGameInfo {
    path: *const c_char,
    data: *const c_void,
    size: usize,
    meta: *const c_char,
}

#[repr(C)]
struct RetroVariable {
    key: *const c_char,
    value: *const c_char,
}

type EnvironmentCallback = unsafe extern "C" fn(c_uint, *mut c_void) -> bool;
type VideoCallback = unsafe extern "C" fn(*const c_void, c_uint, c_uint, usize);
type AudioSampleCallback = unsafe extern "C" fn(i16, i16);
type AudioBatchCallback = unsafe extern "C" fn(*const i16, usize) -> usize;
type InputPollCallback = unsafe extern "C" fn();
type InputStateCallback = unsafe extern "C" fn(c_uint, c_uint, c_uint, c_uint) -> i16;

type RetroSetEnvironment = unsafe extern "C" fn(EnvironmentCallback);
type RetroSetVideo = unsafe extern "C" fn(VideoCallback);
type RetroSetAudio = unsafe extern "C" fn(AudioSampleCallback);
type RetroSetAudioBatch = unsafe extern "C" fn(AudioBatchCallback);
type RetroSetInputPoll = unsafe extern "C" fn(InputPollCallback);
type RetroSetInputState = unsafe extern "C" fn(InputStateCallback);
type RetroInit = unsafe extern "C" fn();
type RetroDeinit = unsafe extern "C" fn();
type RetroGetSystemInfo = unsafe extern "C" fn(*mut RetroSystemInfo);
type RetroGetSystemAvInfo = unsafe extern "C" fn(*mut RetroSystemAvInfo);
type RetroLoadGame = unsafe extern "C" fn(*const RetroGameInfo) -> bool;
type RetroUnloadGame = unsafe extern "C" fn();
type RetroReset = unsafe extern "C" fn();
type RetroRun = unsafe extern "C" fn();
type RetroSerializeSize = unsafe extern "C" fn() -> usize;
type RetroSerialize = unsafe extern "C" fn(*mut c_void, usize) -> bool;
type RetroUnserialize = unsafe extern "C" fn(*const c_void, usize) -> bool;
type RetroGetMemoryData = unsafe extern "C" fn(c_uint) -> *mut c_void;
type RetroGetMemorySize = unsafe extern "C" fn(c_uint) -> usize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreIdentity {
    pub library_name: String,
    pub library_version: String,
    pub valid_extensions: String,
    pub need_fullpath: bool,
    pub fps: f64,
    pub sample_rate_hz: u32,
}

pub fn libretro_capability_manifest(identity: &CoreIdentity) -> RuntimeCapabilityManifest {
    let mut capabilities = RuntimeCapabilities::core_baseline();

    // The libretro adapter exposes these APIs, but generic cores are not
    // promoted to QUALIFIED merely because the ABI symbols exist.
    capabilities.state_snapshots = CapabilityStatus::Supported;
    capabilities.persistent_save_data = CapabilityStatus::Supported;

    let is_pinned_sameboy_profile =
        identity.library_name.eq_ignore_ascii_case("SameBoy")
            && identity.library_version == SAMEBOY_VERSION;

    let mut manifest = RuntimeCapabilityManifest::new(
        identity.library_name.clone(),
        Some(identity.library_version.clone()),
        LIBRETRO_ADAPTER_ID,
        RuntimeExecutionModel::EmbeddedFrameCore,
        capabilities,
    );

    if is_pinned_sameboy_profile {
        manifest.capabilities.frame_step = CapabilityStatus::Qualified;
        manifest.capabilities.rendered_framebuffer = CapabilityStatus::Qualified;
        manifest.capabilities.governed_actions = CapabilityStatus::Qualified;
        manifest.capabilities.state_snapshots = CapabilityStatus::Qualified;
        manifest.capabilities.exact_replay = CapabilityStatus::Qualified;
        manifest = manifest.with_qualification_profile(RuntimeQualificationProfile {
            profile_id: SAMEBOY_QUALIFICATION_PROFILE_ID.into(),
            source_revision: Some(SAMEBOY_SOURCE_REVISION.into()),
            binary_evidence_required: true,
        });
    }

    manifest
}

pub struct LibretroCore {
    _library: Library,
    core_path: PathBuf,
    identity: CoreIdentity,
    av: RetroSystemAvInfo,
    deinit: RetroDeinit,
    unload_game: RetroUnloadGame,
    get_av: RetroGetSystemAvInfo,
    load_game_fn: RetroLoadGame,
    reset_fn: RetroReset,
    run_fn: RetroRun,
    serialize_size_fn: RetroSerializeSize,
    serialize_fn: RetroSerialize,
    unserialize_fn: RetroUnserialize,
    get_memory_data_fn: RetroGetMemoryData,
    get_memory_size_fn: RetroGetMemorySize,
    loaded: bool,
    game_bytes: Vec<u8>,
    game_path: Option<CString>,
    input_mask: u16,
    frame: u64,
}

unsafe fn load_symbol<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, CoreError> {
    lib.get::<T>(name)
        .map(|s| *s)
        .map_err(|e| CoreError::Runtime(format!("missing libretro symbol {}: {e}", String::from_utf8_lossy(name))))
}

impl LibretroCore {
    pub fn open(
        core_path: impl AsRef<Path>,
        system_dir: impl AsRef<Path>,
        save_dir: impl AsRef<Path>,
    ) -> Result<Self, CoreError> {
        let core_path = core_path.as_ref().to_path_buf();
        let library = unsafe { Library::new(&core_path) }
            .map_err(|e| CoreError::Runtime(format!("cannot load {}: {e}", core_path.display())))?;

        unsafe {
            let set_env: RetroSetEnvironment = load_symbol(&library, b"retro_set_environment\0")?;
            let set_video: RetroSetVideo = load_symbol(&library, b"retro_set_video_refresh\0")?;
            let set_audio: RetroSetAudio = load_symbol(&library, b"retro_set_audio_sample\0")?;
            let set_audio_batch: RetroSetAudioBatch = load_symbol(&library, b"retro_set_audio_sample_batch\0")?;
            let set_input_poll: RetroSetInputPoll = load_symbol(&library, b"retro_set_input_poll\0")?;
            let set_input_state: RetroSetInputState = load_symbol(&library, b"retro_set_input_state\0")?;
            let init: RetroInit = load_symbol(&library, b"retro_init\0")?;
            let deinit: RetroDeinit = load_symbol(&library, b"retro_deinit\0")?;
            let get_info: RetroGetSystemInfo = load_symbol(&library, b"retro_get_system_info\0")?;
            let get_av: RetroGetSystemAvInfo = load_symbol(&library, b"retro_get_system_av_info\0")?;
            let load_game_fn: RetroLoadGame = load_symbol(&library, b"retro_load_game\0")?;
            let unload_game: RetroUnloadGame = load_symbol(&library, b"retro_unload_game\0")?;
            let reset_fn: RetroReset = load_symbol(&library, b"retro_reset\0")?;
            let run_fn: RetroRun = load_symbol(&library, b"retro_run\0")?;
            let serialize_size_fn: RetroSerializeSize = load_symbol(&library, b"retro_serialize_size\0")?;
            let serialize_fn: RetroSerialize = load_symbol(&library, b"retro_serialize\0")?;
            let unserialize_fn: RetroUnserialize = load_symbol(&library, b"retro_unserialize\0")?;
            let get_memory_data_fn: RetroGetMemoryData = load_symbol(&library, b"retro_get_memory_data\0")?;
            let get_memory_size_fn: RetroGetMemorySize = load_symbol(&library, b"retro_get_memory_size\0")?;

            {
                let mut state = callbacks().lock().map_err(|_| CoreError::Runtime("callback state poisoned".into()))?;
                state.system_dir = path_cstring(system_dir.as_ref())?;
                state.save_dir = path_cstring(save_dir.as_ref())?;
                state.video = RawVideo::default();
                state.audio.clear();
                state.input_mask = 0;
                state.variables.clear();
                state.shutdown = false;
            }

            set_env(environment_callback);
            set_video(video_callback);
            set_audio(audio_sample_callback);
            set_audio_batch(audio_batch_callback);
            set_input_poll(input_poll_callback);
            set_input_state(input_state_callback);
            init();

            let mut info = RetroSystemInfo::default();
            get_info(&mut info);
            let mut av = RetroSystemAvInfo::default();
            get_av(&mut av);

            Ok(Self {
                _library: library,
                core_path,
                identity: CoreIdentity {
                    library_name: cstr(info.library_name),
                    library_version: cstr(info.library_version),
                    valid_extensions: cstr(info.valid_extensions),
                    need_fullpath: info.need_fullpath,
                    fps: av.timing.fps,
                    sample_rate_hz: av.timing.sample_rate.round().max(0.0) as u32,
                },
                av,
                deinit,
                unload_game,
                get_av,
                load_game_fn,
                reset_fn,
                run_fn,
                serialize_size_fn,
                serialize_fn,
                unserialize_fn,
                get_memory_data_fn,
                get_memory_size_fn,
                loaded: false,
                game_bytes: Vec::new(),
                game_path: None,
                input_mask: 0,
                frame: 0,
            })
        }
    }

    pub fn identity(&self) -> &CoreIdentity { &self.identity }
    pub fn core_path(&self) -> &Path { &self.core_path }
    pub fn frame_count(&self) -> u64 { self.frame }
    pub fn shutdown_requested(&self) -> bool {
        callbacks().lock().map(|s| s.shutdown).unwrap_or(false)
    }

    pub fn input_mask_snapshot(&self) -> u16 {
        self.input_mask
    }

    pub fn restore_input_mask(&mut self, mask: u16) {
        self.input_mask = mask;
        if let Ok(mut state) = callbacks().lock() {
            state.input_mask = mask;
        }
    }

    pub fn serialize_state(&self) -> Result<Vec<u8>, CoreError> {
        if !self.loaded {
            return Err(CoreError::InvalidState("no game loaded".into()));
        }
        let size = unsafe { (self.serialize_size_fn)() };
        if size == 0 {
            return Err(CoreError::Runtime("core reported zero-byte serialize state".into()));
        }
        let mut bytes = vec![0u8; size];
        let ok = unsafe { (self.serialize_fn)(bytes.as_mut_ptr().cast(), size) };
        if !ok {
            return Err(CoreError::Runtime("core refused state serialization".into()));
        }
        Ok(bytes)
    }

    pub fn restore_state(&mut self, bytes: &[u8], frame: u64) -> Result<(), CoreError> {
        if !self.loaded {
            return Err(CoreError::InvalidState("no game loaded".into()));
        }
        if bytes.is_empty() {
            return Err(CoreError::InvalidState("cannot restore an empty state".into()));
        }
        let ok = unsafe { (self.unserialize_fn)(bytes.as_ptr().cast(), bytes.len()) };
        if !ok {
            return Err(CoreError::Runtime("core refused state restore".into()));
        }
        self.frame = frame;
        self.input_mask = 0;
        if let Ok(mut state) = callbacks().lock() {
            state.input_mask = 0;
            state.audio.clear();
        }
        Ok(())
    }

    pub fn save_ram_size(&self) -> usize {
        if !self.loaded { return 0; }
        unsafe { (self.get_memory_size_fn)(RETRO_MEMORY_SAVE_RAM) }
    }

    pub fn read_save_ram(&self) -> Result<Vec<u8>, CoreError> {
        if !self.loaded {
            return Err(CoreError::InvalidState("no game loaded".into()));
        }
        let size = self.save_ram_size();
        if size == 0 { return Ok(Vec::new()); }
        let data = unsafe { (self.get_memory_data_fn)(RETRO_MEMORY_SAVE_RAM) };
        if data.is_null() {
            return Err(CoreError::Runtime("core exposed save RAM size without data pointer".into()));
        }
        Ok(unsafe { slice::from_raw_parts(data.cast::<u8>(), size) }.to_vec())
    }

    pub fn write_save_ram(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        if !self.loaded {
            return Err(CoreError::InvalidState("no game loaded".into()));
        }
        let size = self.save_ram_size();
        if size == 0 { return Ok(()); }
        if bytes.len() != size {
            return Err(CoreError::InvalidState(format!(
                "save RAM size mismatch: file={} core={size}",
                bytes.len()
            )));
        }
        let data = unsafe { (self.get_memory_data_fn)(RETRO_MEMORY_SAVE_RAM) };
        if data.is_null() {
            return Err(CoreError::Runtime("core exposed save RAM size without data pointer".into()));
        }
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), data.cast::<u8>(), size) };
        Ok(())
    }

    fn apply_actions(&mut self, actions: &[ActionEnvelope]) -> Result<(), CoreError> {
        for event in actions {
            event.validate().map_err(CoreError::InvalidState)?;
            match &event.action {
                ActionKind::Button { button, pressed } => {
                    if let Some(id) = button_id(button) {
                        let bit = 1u16 << id;
                        if *pressed { self.input_mask |= bit; } else { self.input_mask &= !bit; }
                    }
                }
                ActionKind::System { command: SystemCommand::Reset, .. } => unsafe { (self.reset_fn)() },
                _ => {}
            }
        }
        Ok(())
    }
}

impl EmulatorCore for LibretroCore {
    fn core_id(&self) -> &str { &self.identity.library_name }

    fn capability_manifest(&self) -> RuntimeCapabilityManifest {
        libretro_capability_manifest(&self.identity)
    }

    fn load_game(&mut self, image: &GameImage) -> Result<(), CoreError> {
        if !matches!(image.system, SystemId::GameBoy | SystemId::GameBoyColor) {
            return Err(CoreError::UnsupportedImage);
        }
        if self.loaded {
            unsafe { (self.unload_game)() };
            self.loaded = false;
        }
        self.game_bytes = fs::read(&image.path)
            .map_err(|e| CoreError::Runtime(format!("cannot read {}: {e}", image.path.display())))?;
        self.game_path = Some(path_cstring(&image.path)?);
        let info = RetroGameInfo {
            path: self.game_path.as_ref().map_or(ptr::null(), |p| p.as_ptr()),
            data: if self.identity.need_fullpath { ptr::null() } else { self.game_bytes.as_ptr().cast() },
            size: if self.identity.need_fullpath { 0 } else { self.game_bytes.len() },
            meta: ptr::null(),
        };
        let ok = unsafe { (self.load_game_fn)(&info) };
        if !ok { return Err(CoreError::UnsupportedImage); }
        self.loaded = true;
        self.frame = 0;
        unsafe { (self.get_av)(&mut self.av) };
        self.identity.fps = self.av.timing.fps;
        self.identity.sample_rate_hz = self.av.timing.sample_rate.round().max(0.0) as u32;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), CoreError> {
        if !self.loaded { return Err(CoreError::InvalidState("no game loaded".into())); }
        unsafe { (self.reset_fn)() };
        self.frame = 0;
        Ok(())
    }

    fn step_frame(
        &mut self,
        actions: &[ActionEnvelope],
        video: &mut FrameBuffer,
        audio: &mut AudioBuffer,
    ) -> Result<(), CoreError> {
        if !self.loaded { return Err(CoreError::InvalidState("no game loaded".into())); }
        self.apply_actions(actions)?;
        {
            let mut state = callbacks().lock().map_err(|_| CoreError::Runtime("callback state poisoned".into()))?;
            state.input_mask = self.input_mask;
            state.audio.clear();
        }
        unsafe { (self.run_fn)() };
        let state = callbacks().lock().map_err(|_| CoreError::Runtime("callback state poisoned".into()))?;
        *video = convert_video(&state.video, state.format)?;
        audio.sample_rate_hz = self.identity.sample_rate_hz;
        audio.interleaved_stereo_f32 = state.audio.iter().map(|s| *s as f32 / 32768.0).collect();
        drop(state);
        self.frame += 1;
        Ok(())
    }
}

impl Drop for LibretroCore {
    fn drop(&mut self) {
        unsafe {
            if self.loaded { (self.unload_game)(); }
            (self.deinit)();
        }
    }
}

fn button_id(button: &str) -> Option<c_uint> {
    match button.to_ascii_uppercase().as_str() {
        "A" => Some(A), "B" => Some(B), "SELECT" => Some(SELECT), "START" => Some(START),
        "UP" => Some(UP), "DOWN" => Some(DOWN), "LEFT" => Some(LEFT), "RIGHT" => Some(RIGHT),
        _ => None,
    }
}

fn path_cstring(path: &Path) -> Result<CString, CoreError> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| CoreError::Runtime(format!("path contains NUL byte: {}", path.display())))
}

fn cstr(p: *const c_char) -> String {
    if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
}

fn convert_video(raw: &RawVideo, format: PixelFormat) -> Result<FrameBuffer, CoreError> {
    if raw.width == 0 || raw.height == 0 || raw.bytes.is_empty() {
        return Ok(FrameBuffer::default());
    }
    let bpp = if matches!(format, PixelFormat::Xrgb8888) { 4 } else { 2 };
    if raw.pitch < raw.width as usize * bpp || raw.bytes.len() < raw.pitch * raw.height as usize {
        return Err(CoreError::Runtime("core returned invalid video pitch".into()));
    }
    let mut rgba8 = Vec::with_capacity(raw.width as usize * raw.height as usize * 4);
    for y in 0..raw.height as usize {
        let row = &raw.bytes[y * raw.pitch..y * raw.pitch + raw.width as usize * bpp];
        match format {
            PixelFormat::Xrgb8888 => for px in row.chunks_exact(4) {
                let v = u32::from_ne_bytes([px[0], px[1], px[2], px[3]]);
                rgba8.extend_from_slice(&[((v >> 16) & 255) as u8, ((v >> 8) & 255) as u8, (v & 255) as u8, 255]);
            },
            PixelFormat::Rgb565 => for px in row.chunks_exact(2) {
                let v = u16::from_ne_bytes([px[0], px[1]]);
                let r=((v>>11)&31) as u8; let g=((v>>5)&63) as u8; let b=(v&31) as u8;
                rgba8.extend_from_slice(&[(r<<3)|(r>>2),(g<<2)|(g>>4),(b<<3)|(b>>2),255]);
            },
            PixelFormat::ZeroRgb1555 => for px in row.chunks_exact(2) {
                let v = u16::from_ne_bytes([px[0], px[1]]);
                let r=((v>>10)&31) as u8; let g=((v>>5)&31) as u8; let b=(v&31) as u8;
                rgba8.extend_from_slice(&[(r<<3)|(r>>2),(g<<3)|(g>>2),(b<<3)|(b>>2),255]);
            },
        }
    }
    Ok(FrameBuffer { width: raw.width, height: raw.height, rgba8 })
}

unsafe extern "C" fn environment_callback(command: c_uint, data: *mut c_void) -> bool {
    let Ok(mut state) = callbacks().lock() else { return false; };
    match command {
        GET_CAN_DUPE => {
            if data.is_null() { return false; }
            *(data as *mut bool) = true; true
        }
        SHUTDOWN => { state.shutdown = true; true }
        GET_SYSTEM_DIRECTORY => {
            if data.is_null() { return false; }
            *(data as *mut *const c_char) = state.system_dir.as_ptr(); true
        }
        GET_SAVE_DIRECTORY => {
            if data.is_null() { return false; }
            *(data as *mut *const c_char) = state.save_dir.as_ptr(); true
        }
        SET_PIXEL_FORMAT => {
            if data.is_null() { return false; }
            state.format = match *(data as *const c_uint) {
                0 => PixelFormat::ZeroRgb1555, 1 => PixelFormat::Xrgb8888, 2 => PixelFormat::Rgb565, _ => return false
            };
            true
        }
        SET_VARIABLES => {
            if data.is_null() { return false; }
            state.variables.clear();
            let mut p = data as *const RetroVariable;
            while !(*p).key.is_null() {
                let key = CStr::from_ptr((*p).key).to_bytes();
                let raw = if (*p).value.is_null() { &[][..] } else { CStr::from_ptr((*p).value).to_bytes() };
                let default = raw.split(|b| *b == b';').nth(1)
                    .map(|v| v.iter().copied().skip_while(|b| b.is_ascii_whitespace()).take_while(|b| *b != b'|').collect::<Vec<_>>())
                    .unwrap_or_default();
                if let (Ok(k), Ok(v)) = (CString::new(key), CString::new(default)) { state.variables.push((k,v)); }
                p = p.add(1);
            }
            true
        }
        GET_VARIABLE => {
            if data.is_null() { return false; }
            let variable = &mut *(data as *mut RetroVariable);
            if variable.key.is_null() { return false; }
            let key = CStr::from_ptr(variable.key).to_bytes();
            if let Some((_, value)) = state.variables.iter().find(|(k,_)| k.as_bytes()==key) {
                variable.value = value.as_ptr(); true
            } else { variable.value = ptr::null(); false }
        }
        GET_VARIABLE_UPDATE => {
            if data.is_null() { return false; }
            *(data as *mut bool) = false; true
        }
        GET_INPUT_BITMASKS => false,
        GET_CORE_OPTIONS_VERSION => {
            if data.is_null() { return false; }
            *(data as *mut c_uint) = 0; true
        }
        SET_INPUT_DESCRIPTORS | SET_SYSTEM_AV_INFO | SET_CONTROLLER_INFO | SET_GEOMETRY
        | SET_CORE_OPTIONS | SET_CORE_OPTIONS_INTL | SET_CORE_OPTIONS_DISPLAY
        | SET_CORE_OPTIONS_V2 | SET_CORE_OPTIONS_V2_INTL | SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK => true,
        _ => false,
    }
}

unsafe extern "C" fn video_callback(data: *const c_void, width: c_uint, height: c_uint, pitch: usize) {
    if data.is_null() { return; }
    let Ok(mut state) = callbacks().lock() else { return; };
    let len = pitch.saturating_mul(height as usize);
    state.video = RawVideo { width, height, pitch, bytes: slice::from_raw_parts(data.cast::<u8>(), len).to_vec() };
}
unsafe extern "C" fn audio_sample_callback(left: i16, right: i16) {
    if let Ok(mut state)=callbacks().lock() { state.audio.extend_from_slice(&[left,right]); }
}
unsafe extern "C" fn audio_batch_callback(data: *const i16, frames: usize) -> usize {
    if data.is_null() { return 0; }
    if let Ok(mut state)=callbacks().lock() {
        state.audio.extend_from_slice(slice::from_raw_parts(data, frames.saturating_mul(2)));
        frames
    } else { 0 }
}
unsafe extern "C" fn input_poll_callback() {}
unsafe extern "C" fn input_state_callback(port:c_uint, device:c_uint, _index:c_uint, id:c_uint)->i16 {
    if port!=0 || device!=RETRO_DEVICE_JOYPAD || id>15 { return 0; }
    callbacks().lock().map(|s| if s.input_mask & (1u16<<id)!=0 {1} else {0}).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_gameboy_buttons() {
        assert_eq!(button_id("A"), Some(A));
        assert_eq!(button_id("left"), Some(LEFT));
        assert_eq!(button_id("wat"), None);
    }
    #[test]
    fn sameboy_manifest_marks_only_qualified_capabilities() {
        let identity = CoreIdentity {
            library_name: "SameBoy".into(),
            library_version: SAMEBOY_VERSION.into(),
            valid_extensions: "gb|gbc".into(),
            need_fullpath: false,
            fps: 59.7,
            sample_rate_hz: 48_000,
        };
        let manifest = libretro_capability_manifest(&identity);

        assert_eq!(manifest.adapter_id, LIBRETRO_ADAPTER_ID);
        assert_eq!(manifest.execution_model, RuntimeExecutionModel::EmbeddedFrameCore);
        assert_eq!(manifest.capabilities.frame_step, CapabilityStatus::Qualified);
        assert_eq!(manifest.capabilities.rendered_framebuffer, CapabilityStatus::Qualified);
        assert_eq!(manifest.capabilities.governed_actions, CapabilityStatus::Qualified);
        assert_eq!(manifest.capabilities.state_snapshots, CapabilityStatus::Qualified);
        assert_eq!(manifest.capabilities.exact_replay, CapabilityStatus::Qualified);
        assert_eq!(manifest.capabilities.persistent_save_data, CapabilityStatus::Supported);
        assert_eq!(manifest.capabilities.game_detection, CapabilityStatus::Unsupported);
        assert_eq!(manifest.capabilities.external_process_lifecycle, CapabilityStatus::Unsupported);

        let profile = manifest.qualification_profile.expect("sameboy qualification profile");
        assert_eq!(profile.profile_id, SAMEBOY_QUALIFICATION_PROFILE_ID);
        assert_eq!(profile.source_revision.as_deref(), Some(SAMEBOY_SOURCE_REVISION));
        assert!(profile.binary_evidence_required);
    }

    #[test]
    fn generic_libretro_core_does_not_inherit_sameboy_qualification() {
        let identity = CoreIdentity {
            library_name: "OtherCore".into(),
            library_version: "9.9".into(),
            valid_extensions: "rom".into(),
            need_fullpath: false,
            fps: 60.0,
            sample_rate_hz: 48_000,
        };
        let manifest = libretro_capability_manifest(&identity);

        assert_eq!(manifest.capabilities.frame_step, CapabilityStatus::Supported);
        assert_eq!(manifest.capabilities.state_snapshots, CapabilityStatus::Supported);
        assert_eq!(manifest.capabilities.exact_replay, CapabilityStatus::Unsupported);
        assert!(manifest.qualification_profile.is_none());
    }

    #[test]
    fn converts_xrgb8888() {
        let raw=RawVideo{width:1,height:1,pitch:4,bytes:0x00112233u32.to_ne_bytes().to_vec()};
        let frame=convert_video(&raw,PixelFormat::Xrgb8888).unwrap();
        assert_eq!(frame.rgba8, vec![0x11,0x22,0x33,0xff]);
    }
}
