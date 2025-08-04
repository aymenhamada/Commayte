use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub cpu_cores: Option<u32>,
    pub cpu_model: Option<String>,
    pub memory_gb: Option<u32>,
    pub gpu_model: Option<String>,
    pub os_info: Option<String>,
    pub performance_level: PerformanceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceLevel {
    VeryLow,  // Very slow computer - minimal prompts
    Low,      // Slow computer - short prompts
    Medium,   // Average computer - medium prompts
    Good,     // Good computer - longer prompts
    High,     // Fast computer - long prompts
    VeryHigh, // Very fast computer - maximum prompts
}

impl SystemSpecs {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cpu_cores = num_cpus::get() as u32;
        let cpu_model = get_cpu_model()?;
        let memory_gb = get_memory_gb()?;
        let gpu_model = get_gpu_model().ok();
        let os_info = get_os_info()?;
        let performance_level = determine_performance_level(cpu_cores, memory_gb, &cpu_model);

        Ok(SystemSpecs {
            cpu_cores: Some(cpu_cores),
            cpu_model: Some(cpu_model),
            memory_gb: Some(memory_gb),
            gpu_model,
            os_info: Some(os_info),
            performance_level,
        })
    }
    pub fn get_max_total_content(&self) -> usize {
        match self.performance_level {
            PerformanceLevel::VeryLow => 1000,
            PerformanceLevel::Low => 2000,
            PerformanceLevel::Medium => 3000,
            PerformanceLevel::Good => 5000,
            PerformanceLevel::High => 10000,
            PerformanceLevel::VeryHigh => 15000,
        }
    }

    pub fn get_max_file_content(&self) -> usize {
        match self.performance_level {
            PerformanceLevel::VeryLow => 100,
            PerformanceLevel::Low => 200,
            PerformanceLevel::Medium => 300,
            PerformanceLevel::Good => 500,
            PerformanceLevel::High => 1000,
            PerformanceLevel::VeryHigh => 1500,
        }
    }
}

fn get_cpu_model() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("cat").arg("/proc/cpuinfo").output()?;

        let cpuinfo = String::from_utf8(output.stdout)?;
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return Ok(parts[1].trim().to_string());
                }
            }
        }
        Ok("Unknown CPU".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()?;

        let model = String::from_utf8(output.stdout)?;
        Ok(model.trim().to_string())
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic")
            .args(&["cpu", "get", "name", "/format:list"])
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.starts_with("Name=") {
                let model = line.replace("Name=", "").trim().to_string();
                return Ok(model);
            }
        }
        Ok("Unknown CPU".to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok("Unknown CPU".to_string())
    }
}

fn get_memory_gb() -> Result<u32, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("cat").arg("/proc/meminfo").output()?;

        let meminfo = String::from_utf8(output.stdout)?;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    let kb = parts[1].parse::<u32>()?;
                    return Ok(kb / 1024 / 1024); // Convert KB to GB
                }
            }
        }
        Ok(8) // Default fallback
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()?;

        let bytes = String::from_utf8(output.stdout)?.trim().parse::<u64>()?;
        Ok((bytes / 1024 / 1024 / 1024) as u32) // Convert bytes to GB
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic")
            .args(&[
                "computersystem",
                "get",
                "TotalPhysicalMemory",
                "/format:list",
            ])
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.starts_with("TotalPhysicalMemory=") {
                let bytes = line
                    .replace("TotalPhysicalMemory=", "")
                    .trim()
                    .parse::<u64>()?;
                return Ok((bytes / 1024 / 1024 / 1024) as u32); // Convert bytes to GB
            }
        }
        Ok(8) // Default fallback
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok(8) // Default fallback
    }
}

fn get_gpu_model() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        // Try lspci first
        let output = Command::new("lspci").arg("-v").output();

        if let Ok(output) = output {
            let lspci_output = String::from_utf8(output.stdout)?;
            for line in lspci_output.lines() {
                if line.contains("VGA") || line.contains("3D") || line.contains("Display") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() > 2 {
                        return Ok(parts[2].trim().to_string());
                    }
                }
            }
        }

        // Fallback to glxinfo
        let output = Command::new("glxinfo").arg("-B").output();

        if let Ok(output) = output {
            let glx_output = String::from_utf8(output.stdout)?;
            for line in glx_output.lines() {
                if line.starts_with("OpenGL renderer string:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() > 1 {
                        return Ok(parts[1].trim().to_string());
                    }
                }
            }
        }

        Ok("Unknown GPU".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.contains("Chipset Model:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return Ok(parts[1].trim().to_string());
                }
            }
        }
        Ok("Unknown GPU".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("wmic")
            .args(&[
                "path",
                "win32_VideoController",
                "get",
                "name",
                "/format:list",
            ])
            .output()?;

        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.starts_with("Name=") {
                let gpu = line.replace("Name=", "").trim().to_string();
                return Ok(gpu);
            }
        }
        Ok("Unknown GPU".to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok("Unknown GPU".to_string())
    }
}

fn get_os_info() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("cat").arg("/etc/os-release").output()?;

        let os_release = String::from_utf8(output.stdout)?;
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let os_name = line
                    .replace("PRETTY_NAME=", "")
                    .replace("\"", "")
                    .trim()
                    .to_string();
                return Ok(format!("Linux - {}", os_name));
            }
        }
        Ok("Linux".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sw_vers").arg("-productName").output()?;

        let product = String::from_utf8(output.stdout)?.trim().to_string();
        let version_output = Command::new("sw_vers").arg("-productVersion").output()?;

        let version = String::from_utf8(version_output.stdout)?.trim().to_string();
        Ok(format!("macOS {product} {version}"))
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("ver").output()?;

        let version = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(format!("Windows {}", version))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Ok("Unknown OS".to_string())
    }
}

fn determine_performance_level(
    cpu_cores: u32,
    memory_gb: u32,
    cpu_model: &str,
) -> PerformanceLevel {
    // Check for very high-end CPU models
    let is_very_high_end_cpu = cpu_model.to_lowercase().contains("i9")
        || cpu_model.to_lowercase().contains("ryzen 9")
        || cpu_model.to_lowercase().contains("m2 pro")
        || cpu_model.to_lowercase().contains("m2 max")
        || cpu_model.to_lowercase().contains("m3 pro")
        || cpu_model.to_lowercase().contains("m3 max")
        || cpu_model.to_lowercase().contains("threadripper");

    // Check for high-end CPU models
    let is_high_end_cpu = cpu_model.to_lowercase().contains("i7")
        || cpu_model.to_lowercase().contains("ryzen 7")
        || cpu_model.to_lowercase().contains("m1")
        || cpu_model.to_lowercase().contains("m2")
        || cpu_model.to_lowercase().contains("m3");

    // Check for good CPU models
    let is_good_cpu = cpu_model.to_lowercase().contains("i5")
        || cpu_model.to_lowercase().contains("ryzen 5")
        || cpu_model.to_lowercase().contains("fx");

    // Check for low-end CPU models
    let is_low_end_cpu = cpu_model.to_lowercase().contains("celeron")
        || cpu_model.to_lowercase().contains("atom")
        || cpu_model.to_lowercase().contains("pentium")
        || cpu_model.to_lowercase().contains("athlon")
        || cpu_model.to_lowercase().contains("sempron");

    // Check for very low-end CPU models (more specific patterns)
    let is_very_low_end_cpu =
        cpu_model.to_lowercase().contains("atom") || cpu_model.to_lowercase().contains("sempron");

    // Determine performance level based on specs
    if cpu_cores >= 12 && memory_gb >= 32 && is_very_high_end_cpu {
        PerformanceLevel::VeryHigh
    } else if cpu_cores >= 8 && memory_gb >= 16 && is_high_end_cpu {
        PerformanceLevel::High
    } else if cpu_cores >= 6 && memory_gb >= 12 && (is_good_cpu || is_high_end_cpu) {
        PerformanceLevel::Good
    } else if cpu_cores >= 4 && memory_gb >= 8 && !is_low_end_cpu {
        PerformanceLevel::Medium
    } else if cpu_cores >= 2 && memory_gb >= 4 && !is_very_low_end_cpu {
        PerformanceLevel::Low
    } else {
        PerformanceLevel::VeryLow
    }
}

pub fn get_system_info() -> Result<SystemSpecs, Box<dyn std::error::Error>> {
    SystemSpecs::new()
}
