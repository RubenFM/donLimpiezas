//! Interpretación de las líneas de comandos que los programas registran en Windows.

/// Expande `%VARIABLE%` con `env`; las variables desconocidas se dejan tal cual.
pub fn expand_env(s: &str, env: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let name = &after[..end];
        match env(name).filter(|_| !name.is_empty()) {
            Some(value) => out.push_str(&value),
            None => out.push_str(&rest[start..start + end + 2]),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

pub fn system_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Separa ejecutable y argumentos: el primer token entre comillas o, sin
/// comillas, todo hasta el primer `.exe` (las rutas pueden tener espacios).
/// Las variables de entorno del ejecutable se expanden.
pub fn split(command: &str, env: impl Fn(&str) -> Option<String>) -> Option<(String, String)> {
    let command = command.trim();
    let (path, args) = match command.strip_prefix('"') {
        Some(rest) => {
            let end = rest.find('"')?;
            (&rest[..end], &rest[end + 1..])
        }
        None => match command.to_ascii_lowercase().find(".exe") {
            Some(i) => command.split_at(i + 4),
            None => command.split_once(char::is_whitespace).unwrap_or((command, "")),
        },
    };
    let path = expand_env(path, env);
    (!path.is_empty()).then(|| (path, args.trim().to_string()))
}

/// Código de producto (`{GUID}`) de un comando de `msiexec`, si lo es.
pub fn msi_product_code(command: &str) -> Option<String> {
    if !command.to_ascii_lowercase().contains("msiexec") {
        return None;
    }
    let start = command.find('{')?;
    let end = start + command[start..].find('}')?;
    Some(command[start..=end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str) -> Option<String> {
        match name {
            "ProgramFiles" => Some(r"C:\Program Files".into()),
            _ => None,
        }
    }

    fn exe(command: &str) -> Option<String> {
        split(command, env).map(|(exe, _)| exe)
    }

    #[test]
    fn ejecutable_entre_comillas() {
        assert_eq!(
            split(r#""C:\Program Files\X\x.exe" --min"#, env),
            Some((r"C:\Program Files\X\x.exe".into(), "--min".into()))
        );
    }

    #[test]
    fn ejecutable_sin_comillas() {
        assert_eq!(split(r"C:\Tools\y.exe /s", env), Some((r"C:\Tools\y.exe".into(), "/s".into())));
        assert_eq!(exe(r"C:\Program Files\Y\Y.EXE -silent").as_deref(), Some(r"C:\Program Files\Y\Y.EXE"));
        assert_eq!(split(r"C:\Tools\run.bat now", env), Some((r"C:\Tools\run.bat".into(), "now".into())));
    }

    #[test]
    fn ejecutable_con_variables() {
        assert_eq!(exe(r"%ProgramFiles%\Z\z.exe").as_deref(), Some(r"C:\Program Files\Z\z.exe"));
        assert_eq!(expand_env(r"%Desconocida%\a.exe", env), r"%Desconocida%\a.exe");
        assert_eq!(expand_env("100% seguro", env), "100% seguro");
    }

    #[test]
    fn codigo_de_producto_msi() {
        let guid = "{23170F69-40C1-2702-2408-000001000000}";
        assert_eq!(msi_product_code(&format!("MsiExec.exe /I{guid}")).as_deref(), Some(guid));
        assert_eq!(msi_product_code(&format!("msiexec /x {guid} /qb")).as_deref(), Some(guid));
        assert_eq!(msi_product_code(r#""C:\Program Files\7-Zip\Uninstall.exe""#), None);
    }
}
