/// Need to add env variables at program startup if use jemalloc memory profile
/// export _RJEM_MALLOC_CONF=prof:true,prof_active:false
/// Or
/// export MALLOC_CONF=prof:true,prof_active:false

pub mod http_server {
    #[allow(unused_imports)]
    use std::collections::HashMap;
    use warp::Filter;

    fn help(port: u16) -> String {
        let help = format!("curl localhost:{}/help\r\n", port);

        #[cfg(feature = "cpu")]
            let help = format!(
            "{}{}",
            help,
            [
                format!("curl localhost:{}/cpu/active", port),
                format!("curl localhost:{}/cpu/dump", port),
                format!("curl localhost:{}/cpu/deactive\r\n", port),
            ]
                .join("\r\n")
        );

        #[cfg(feature = "jemalloc")]
            let help = format!(
            "{}{}",
            help,
            [
                format!("curl localhost:{}/mem/active", port),
                format!("curl localhost:{}/mem/dump", port),
                format!("curl localhost:{}/mem/deactive\r\n", port),
            ]
                .join("\r\n")
        );
        help
    }

    pub async fn start(port: u16) {
        let root = warp::path::end().map(|| "Hello, prof!\r\n");

        #[cfg(feature = "cpu")]
        {
            let cpu = warp::path!("cpu" / String).and(warp::query()).map(
                |path: String, param: HashMap<String, String>| {
                    if let Err(e) = handle_cpu(&path, param) {
                        format!("occur err: {:?}\r\n", e)
                    } else {
                        "success\r\n".to_owned()
                    }
                },
            );
            root.or(cpu);
        }

        #[cfg(feature = "jemalloc")]
        {
            let mem = warp::path!("mem" / String).map(|path: String| {
                if let Err(e) = handle_mem(&path) {
                    format!("occur err: {:?}\r\n", e)
                } else {
                    "success\r\n".to_owned()
                }
            });
            root.or(mem);
        }

        let help = warp::path("help").map(move || help(port));
        let root = root.or(help);
        let server = warp::get().and(root);
        warp::serve(server).run(([127, 0, 0, 1], port)).await;
    }

    #[cfg(feature = "cpu")]
    fn handle_cpu(path: &str, param: HashMap<String, String>) -> anyhow::Result<()> {
        let frequency = param
            .get("frequency")
            .map(|f| f.parse::<i32>().unwrap_or(100))
            .unwrap_or(100);

        match path {
            "active" => crate::cpu::active(frequency)?,

            "deactive" => crate::cpu::deactive()?,

            "dump" => crate::cpu::dump("cpu_prof")?,

            _ => {}
        }
        Ok(())
    }

    #[cfg(feature = "jemalloc")]
    fn handle_mem(path: &str) -> anyhow::Result<()> {
        match path {
            "active" => crate::jemalloc_mem::active()?,

            "deactive" => crate::jemalloc_mem::deactive()?,

            "dump" => crate::jemalloc_mem::dump("mem_prof")?,

            _ => {}
        }
        Ok(())
    }
}

#[cfg(feature = "jemalloc")]
pub mod jemalloc_mem {
    use crate::date_suffix;
    use std::{ffi::CString, os::raw::c_char};

    // C string should end with a '\0'.
    const PROF_ACTIVE: &[u8] = b"prof.active\0";
    const PROF_DUMP: &[u8] = b"prof.dump\0";

    pub fn active() -> anyhow::Result<()> {
        unsafe {
            jemalloc_ctl::raw::update(PROF_ACTIVE, true)?;
        }
        Ok(())
    }

    pub fn deactive() -> anyhow::Result<()> {
        unsafe {
            jemalloc_ctl::raw::update(PROF_ACTIVE, false)?;
        }
        Ok(())
    }

    pub fn dump(file_name: &str) -> anyhow::Result<()> {
        let file_path = format!("./{}.{}.out", file_name, date_suffix());
        let mut bytes = CString::new(file_path)?.into_bytes_with_nul();
        let ptr = bytes.as_mut_ptr() as *mut c_char;
        unsafe {
            jemalloc_ctl::raw::write(PROF_DUMP, ptr)?;
        }
        Ok(())
    }
}

#[cfg(feature = "cpu")]
pub mod cpu {
    use std::fs::File;

    use anyhow::bail;
    use parking_lot::RwLock;
    use pprof::ProfilerGuard;

    use crate::date_suffix;

    lazy_static::lazy_static! {
        static ref CPU_PROF : RwLock<Option<ProfilerGuard<'static>>> = RwLock::new(None);
    }

    pub fn active(frequency: i32) -> anyhow::Result<()> {
        let cpu_prof = &mut *CPU_PROF.write();
        if cpu_prof.is_some() {
            bail!("cpu prof is activated, no need to active again");
        }
        let guard = ProfilerGuard::new(frequency)?;
        cpu_prof.replace(guard);
        Ok(())
    }

    pub fn deactive() -> anyhow::Result<()> {
        CPU_PROF.write().take();
        Ok(())
    }

    pub fn dump(file_name: &str) -> anyhow::Result<()> {
        let file_path = format!("./{}.{}.svg", file_name, date_suffix());
        let cpu_prof = &*CPU_PROF.read();
        match cpu_prof {
            Some(profiler) => {
                let report = profiler.report().build()?;
                let file = File::create(file_path)?;
                report.flamegraph(file)?;
                Ok(())
            }
            None => bail!("cpu prof is not activated"),
        }
    }
}

use chrono::Local;

#[allow(dead_code)]
fn date_suffix() -> String {
    Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}
