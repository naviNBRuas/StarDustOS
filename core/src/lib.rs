//! StarDust core: turns a (tenant, runtime) pair into a sandboxed launch command.
//! Principle: a runtime only ever sees ITS tenant's directories. Steam and
//! emulators share hardware, never writable state.

use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Trust { Verified, Untrusted }

#[derive(Debug, Clone)]
pub struct Tenant {
    pub id: String,
    pub root: String,
    pub network: bool,
    /// Subdirectories of `root` exposed to the runtime.
    pub visible_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub id: String,
    pub exec: Vec<String>,
    pub network: bool,
    pub trust: Trust,
}

/// Build the bubblewrap argv. Network is allowed only if BOTH tenant and runtime
/// allow it, and untrusted runtimes never get it.
pub fn bwrap_args(t: &Tenant, r: &Runtime) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "bwrap".into(), "--die-with-parent".into(), "--unshare-pid".into(),
        "--unshare-ipc".into(), "--new-session".into(),
        "--ro-bind".into(), "/usr".into(), "/usr".into(),
        "--symlink".into(), "usr/lib".into(), "/lib".into(),
        "--symlink".into(), "usr/lib64".into(), "/lib64".into(),
        "--symlink".into(), "usr/bin".into(), "/bin".into(),
        "--proc".into(), "/proc".into(),
        "--tmpfs".into(), "/tmp".into(),
        // GPU access. Audio/Wayland/input sockets are added by the broker per launch.
        "--dev-bind".into(), "/dev/dri".into(), "/dev/dri".into(),
    ];
    if !(t.network && r.network && r.trust == Trust::Verified) {
        a.push("--unshare-net".into());
    }
    for p in &t.visible_paths {
        let host = Path::new(&t.root).join(p).to_string_lossy().to_string();
        // BIOS is read-only; everything else is writable within the tenant.
        let flag = if p == "bios" { "--ro-bind" } else { "--bind" };
        a.push(flag.into()); a.push(host.clone()); a.push(host);
    }
    a.push("--".into());
    a.extend(r.exec.iter().cloned());
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tenant(net: bool) -> Tenant {
        Tenant { id: "retro".into(), root: "/games/libraries/retro".into(),
                 network: net, visible_paths: vec!["roms".into(), "bios".into()] }
    }
    fn rt(net: bool, trust: Trust) -> Runtime {
        Runtime { id: "x".into(), exec: vec!["emu".into()], network: net, trust }
    }
    #[test]
    fn no_network_by_default() {
        assert!(bwrap_args(&tenant(false), &rt(false, Trust::Verified)).contains(&"--unshare-net".into()));
    }
    #[test]
    fn untrusted_never_gets_network() {
        assert!(bwrap_args(&tenant(true), &rt(true, Trust::Untrusted)).contains(&"--unshare-net".into()));
    }
    #[test]
    fn bios_read_only_and_other_tenants_invisible() {
        let a = bwrap_args(&tenant(false), &rt(false, Trust::Verified)).join(" ");
        assert!(a.contains("--ro-bind /games/libraries/retro/bios"));
        assert!(!a.contains("steam"));
    }
}
