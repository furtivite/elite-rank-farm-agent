use std::path::PathBuf;

const FILE_NAME: &str = "read-token.bin";

fn directory() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata).join("elite-agent");
    }

    crate::dir::executable_dir()
}

fn path() -> PathBuf {
    directory().join(FILE_NAME)
}

pub fn load() -> String {
    let Ok(stored) = std::fs::read(path()) else {
        return String::new();
    };

    match decrypt(&stored) {
        Ok(token) => String::from_utf8(token)
            .map(|value| value.trim().to_owned())
            .unwrap_or_default(),
        Err(_) => String::new(),
    }
}

pub fn save(token: &str) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        return clear();
    }

    let directory = directory();
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?;

    let encrypted = encrypt(token.as_bytes())?;
    let path = path();
    std::fs::write(&path, encrypted).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn clear() -> Result<(), String> {
    let path = path();
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

#[cfg(windows)]
fn encrypt(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};

    let input = CRYPT_INTEGER_BLOB {
        cbData: plain.len() as u32,
        pbData: plain.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptProtectData(
            &input,
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut output,
        )
    };

    if ok == 0 {
        return Err(agent_core::tr!(
            "could not encrypt the key with Windows",
            "не удалось зашифровать ключ средствами Windows"
        ));
    }

    Ok(take_blob(output))
}

#[cfg(windows)]
fn decrypt(blob: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    let input = CRYPT_INTEGER_BLOB {
        cbData: blob.len() as u32,
        pbData: blob.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut output,
        )
    };

    if ok == 0 {
        return Err(agent_core::tr!(
            "could not decrypt the saved key",
            "не удалось расшифровать сохранённый ключ"
        ));
    }

    Ok(take_blob(output))
}

#[cfg(windows)]
fn take_blob(blob: windows_sys::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB) -> Vec<u8> {
    use windows_sys::Win32::Foundation::LocalFree;

    let data = unsafe { std::slice::from_raw_parts(blob.pbData, blob.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(blob.pbData as _);
    }

    data
}

#[cfg(not(windows))]
fn encrypt(plain: &[u8]) -> Result<Vec<u8>, String> {
    Ok(plain.to_vec())
}

#[cfg(not(windows))]
fn decrypt(blob: &[u8]) -> Result<Vec<u8>, String> {
    Ok(blob.to_vec())
}
