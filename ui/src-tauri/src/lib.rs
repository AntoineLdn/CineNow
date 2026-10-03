// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ------------------------------------------------------------------- Keychain
//
// Le token JWT est confié au gestionnaire de secrets natif de l'OS (Credential
// Manager sous Windows, Keychain sous macOS, Secret Service sous Linux). Il
// n'est donc jamais écrit en clair sur le disque, ni exposé au XSS comme le
// serait localStorage.

const KEYCHAIN_SERVICE: &str = "cinenow";
const KEYCHAIN_USER: &str = "auth-token";

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_USER).map_err(|e| e.to_string())
}

/// Écrit (ou remplace) le token dans le keychain.
#[tauri::command]
fn save_token(token: String) -> Result<(), String> {
    entry()?.set_password(&token).map_err(|e| e.to_string())
}

/// Lit le token. L'absence d'entrée renvoie `None` : ce n'est pas une erreur,
/// juste un utilisateur qui n'a pas encore de session.
#[tauri::command]
fn get_token() -> Result<Option<String>, String> {
    match entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Supprime le token. Supprimer une entrée déjà absente est un succès, pour que
/// la déconnexion reste idempotente.
#[tauri::command]
fn delete_token() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            save_token,
            get_token,
            delete_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
