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
            PerformanceLevel::VeryLow => 1500,
            PerformanceLevel::Low => 2000,
            PerformanceLevel::Medium => 3000,
            PerformanceLevel::Good => 4000,
            PerformanceLevel::High => 5000,
            PerformanceLevel::VeryHigh => 6000,
        }
    }

    pub fn get_max_file_content(&self) -> usize {
        match self.performance_level {
            PerformanceLevel::VeryLow => 200,
            PerformanceLevel::Low => 500,
            PerformanceLevel::Medium => 700,
            PerformanceLevel::Good => 1000,
            PerformanceLevel::High => 1200,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_level_determination() {
        // Test very high-end system
        let very_high_end = determine_performance_level(16, 64, "Intel Core i9-13900K");
        assert!(matches!(very_high_end, PerformanceLevel::VeryHigh));

        // Test high-end system
        let high_end = determine_performance_level(8, 16, "Intel Core i7-10700K");
        assert!(matches!(high_end, PerformanceLevel::High));

        // Test good system
        let good = determine_performance_level(6, 12, "Intel Core i5-10600K");
        assert!(matches!(good, PerformanceLevel::Good));

        // Test medium system
        let medium = determine_performance_level(4, 8, "Intel Core i5-8400");
        assert!(matches!(medium, PerformanceLevel::Medium));

        // Test low-end system
        let low_end = determine_performance_level(2, 4, "Intel Celeron N4000");
        println!("Low-end test result: {:?}", low_end);
        assert!(matches!(low_end, PerformanceLevel::Low));

        // Test very low-end system
        let very_low_end = determine_performance_level(1, 2, "Intel Atom N270");
        assert!(matches!(very_low_end, PerformanceLevel::VeryLow));
    }

    #[test]
    fn test_prompt_length_recommendations() {
        // Test very high-end specs
        let very_high_specs = SystemSpecs {
            cpu_cores: Some(16),
            cpu_model: Some("Intel Core i9".to_string()),
            memory_gb: Some(64),
            gpu_model: Some("RTX 4090".to_string()),
            os_info: Some("Linux".to_string()),
            performance_level: PerformanceLevel::VeryHigh,
        };

        assert_eq!(very_high_specs.get_recommended_prompt_length(), 6000);
        assert_eq!(very_high_specs.get_recommended_max_tokens(), 3000);

        // Test high-end specs
        let high_specs = SystemSpecs {
            cpu_cores: Some(8),
            cpu_model: Some("Intel Core i7".to_string()),
            memory_gb: Some(16),
            gpu_model: Some("RTX 3080".to_string()),
            os_info: Some("Linux".to_string()),
            performance_level: PerformanceLevel::High,
        };

        assert_eq!(high_specs.get_recommended_prompt_length(), 4000);
        assert_eq!(high_specs.get_recommended_max_tokens(), 2000);

        // Test very low-end specs
        let very_low_specs = SystemSpecs {
            cpu_cores: Some(1),
            cpu_model: Some("Intel Atom".to_string()),
            memory_gb: Some(2),
            gpu_model: Some("Intel HD Graphics".to_string()),
            os_info: Some("Linux".to_string()),
            performance_level: PerformanceLevel::VeryLow,
        };

        assert_eq!(very_low_specs.get_recommended_prompt_length(), 500);
        assert_eq!(very_low_specs.get_recommended_max_tokens(), 250);
    }
}
