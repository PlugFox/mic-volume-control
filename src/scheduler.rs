use anyhow::{Context, Result};
use windows::{
    Win32::Foundation::*, Win32::System::Com::*, Win32::System::TaskScheduler::*, core::*,
};

const TASK_NAME: &str = "MicrophoneVolumeControl";
const TASK_FOLDER: &str = "\\";

pub struct TaskScheduler {
    service: ITaskService,
}

impl TaskScheduler {
    pub fn new() -> Result<Self> {
        unsafe {
            let service: ITaskService =
                CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
                    .context("Failed to create TaskScheduler instance")?;

            service
                .Connect(None, None, None, None)
                .context("Failed to connect to Task Scheduler service")?;

            Ok(Self { service })
        }
    }

    pub fn register_task(&self, target_volume: f32, interval_minutes: u32) -> Result<()> {
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

        // Create VBScript wrapper to run without console window
        let vbs_path = Self::create_vbs_wrapper(&exe_path, target_volume)?;

        unsafe {
            let root_folder = self
                .service
                .GetFolder(&BSTR::from(TASK_FOLDER))
                .context("Failed to get task folder")?;

            // Delete existing task if it exists
            let _ = root_folder.DeleteTask(&BSTR::from(TASK_NAME), 0);

            // Create new task definition
            let task_definition = self
                .service
                .NewTask(0)
                .context("Failed to create new task definition")?;

            // Set registration info
            let reg_info = task_definition
                .RegistrationInfo()
                .context("Failed to get registration info")?;
            reg_info
                .SetAuthor(&BSTR::from("MicVolumeControl"))
                .context("Failed to set author")?;
            reg_info
                .SetDescription(&BSTR::from(
                    "Automatically sets microphone volume to configured level",
                ))
                .context("Failed to set description")?;

            // Set principal (run with highest privileges)
            let principal = task_definition
                .Principal()
                .context("Failed to get principal")?;
            principal
                .SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN)
                .context("Failed to set logon type")?;
            principal
                .SetRunLevel(TASK_RUNLEVEL_HIGHEST)
                .context("Failed to set run level")?;

            // Create triggers
            let triggers = task_definition
                .Triggers()
                .context("Failed to get triggers collection")?;

            // 1. Logon trigger - run at login with delay
            let logon_trigger = triggers
                .Create(TASK_TRIGGER_LOGON)
                .context("Failed to create logon trigger")?;
            logon_trigger
                .SetEnabled(VARIANT_TRUE)
                .context("Failed to enable logon trigger")?;

            let logon_trigger_cast: ILogonTrigger = logon_trigger
                .cast()
                .context("Failed to cast to ILogonTrigger")?;
            logon_trigger_cast
                .SetDelay(&BSTR::from("PT1M"))
                .context("Failed to set logon delay")?;

            // 2. Time trigger - repeat every N minutes
            let time_trigger = triggers
                .Create(TASK_TRIGGER_TIME)
                .context("Failed to create time trigger")?;
            time_trigger
                .SetEnabled(VARIANT_TRUE)
                .context("Failed to enable time trigger")?;

            let time_trigger_cast: ITimeTrigger = time_trigger
                .cast()
                .context("Failed to cast to ITimeTrigger")?;

            // Start immediately (or at next boot)
            time_trigger_cast
                .SetStartBoundary(&BSTR::from("2025-01-01T00:00:00"))
                .context("Failed to set start boundary")?;

            // Set repetition pattern
            let repetition = time_trigger_cast
                .Repetition()
                .context("Failed to get repetition pattern")?;

            // Format: PT5M for 5 minutes, PT1H for 1 hour, etc.
            let interval_str = format!("PT{}M", interval_minutes);
            repetition
                .SetInterval(&BSTR::from(interval_str))
                .context("Failed to set repetition interval")?;

            // Run indefinitely
            repetition
                .SetDuration(&BSTR::from(""))
                .context("Failed to set duration")?;

            // Create action (start program)
            let actions = task_definition
                .Actions()
                .context("Failed to get actions collection")?;
            actions
                .SetContext(&BSTR::from("Author"))
                .context("Failed to set actions context")?;

            let action = actions
                .Create(TASK_ACTION_EXEC)
                .context("Failed to create exec action")?;

            let exec_action: IExecAction =
                action.cast().context("Failed to cast to IExecAction")?;

            // Use wscript.exe to run VBScript wrapper (no console window)
            exec_action
                .SetPath(&BSTR::from("wscript.exe"))
                .context("Failed to set executable path")?;

            // Pass VBScript path as argument with //B flag (batch mode, no UI)
            let vbs_path_str = vbs_path
                .to_str()
                .context("Failed to convert VBScript path to string")?;
            let args = format!("//B //Nologo \"{}\"", vbs_path_str);
            exec_action
                .SetArguments(&BSTR::from(args))
                .context("Failed to set arguments")?;

            // Set working directory
            if let Some(parent) = exe_path.parent() {
                let parent_str = parent
                    .to_str()
                    .context("Failed to convert working directory path to string")?;
                exec_action
                    .SetWorkingDirectory(&BSTR::from(parent_str))
                    .context("Failed to set working directory")?;
            }

            // Set task settings
            let settings = task_definition
                .Settings()
                .context("Failed to get task settings")?;

            settings
                .SetEnabled(VARIANT_TRUE)
                .context("Failed to enable task")?;
            settings
                .SetStartWhenAvailable(VARIANT_TRUE)
                .context("Failed to set start when available")?;
            settings
                .SetDisallowStartIfOnBatteries(VARIANT_FALSE)
                .context("Failed to set battery setting")?;
            settings
                .SetStopIfGoingOnBatteries(VARIANT_FALSE)
                .context("Failed to set stop on battery setting")?;
            settings
                .SetAllowDemandStart(VARIANT_TRUE)
                .context("Failed to set allow demand start")?;
            settings
                .SetExecutionTimeLimit(&BSTR::from("PT5M"))
                .context("Failed to set execution time limit")?;
            settings
                .SetMultipleInstances(TASK_INSTANCES_IGNORE_NEW)
                .context("Failed to set multiple instances policy")?;

            // Run hidden without showing window
            settings
                .SetHidden(VARIANT_TRUE)
                .context("Failed to set hidden mode")?;

            // Don't wake computer to run task
            settings
                .SetWakeToRun(VARIANT_FALSE)
                .context("Failed to set wake to run")?;

            // Run task in background with no UI
            settings
                .SetPriority(7) // NORMAL_PRIORITY_CLASS
                .context("Failed to set priority")?;

            // Register the task
            root_folder
                .RegisterTaskDefinition(
                    &BSTR::from(TASK_NAME),
                    &task_definition,
                    TASK_CREATE_OR_UPDATE.0,
                    None,
                    None,
                    TASK_LOGON_INTERACTIVE_TOKEN,
                    None,
                )
                .context("Failed to register task definition")?;

            Ok(())
        }
    }

    pub fn unregister_task(&self) -> Result<()> {
        unsafe {
            let root_folder = self
                .service
                .GetFolder(&BSTR::from(TASK_FOLDER))
                .context("Failed to get task folder")?;

            root_folder
                .DeleteTask(&BSTR::from(TASK_NAME), 0)
                .context("Failed to delete task")?;
        }

        // Clean up VBScript wrapper file
        Self::cleanup_vbs_wrapper()?;

        Ok(())
    }

    pub fn is_registered(&self) -> bool {
        unsafe {
            match self.service.GetFolder(&BSTR::from(TASK_FOLDER)) {
                Ok(folder) => folder.GetTask(&BSTR::from(TASK_NAME)).is_ok(),
                Err(_) => false,
            }
        }
    }

    fn create_vbs_wrapper(
        exe_path: &std::path::Path,
        target_volume: f32,
    ) -> Result<std::path::PathBuf> {
        use std::io::Write;

        // Get application data directory
        let app_data =
            std::env::var("APPDATA").context("APPDATA environment variable not found")?;
        let mut vbs_dir = std::path::PathBuf::from(app_data);
        vbs_dir.push("mic-volume-control");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&vbs_dir).context("Failed to create VBS directory")?;

        let vbs_path = vbs_dir.join("run-silent.vbs");

        // Create VBScript that runs exe without window
        let exe_path_str = exe_path
            .to_str()
            .context("Failed to convert exe path to string")?;
        let volume_percent = (target_volume * 100.0) as u8;

        let vbs_content = format!(
            r#"Set WshShell = CreateObject("WScript.Shell")
WshShell.Run """{}"" volume {}", 0, True
"#,
            exe_path_str, volume_percent
        );

        let mut file = std::fs::File::create(&vbs_path).context("Failed to create VBS file")?;
        file.write_all(vbs_content.as_bytes())
            .context("Failed to write VBS content")?;

        Ok(vbs_path)
    }

    fn cleanup_vbs_wrapper() -> Result<()> {
        let app_data =
            std::env::var("APPDATA").context("APPDATA environment variable not found")?;
        let mut vbs_path = std::path::PathBuf::from(app_data);
        vbs_path.push("mic-volume-control");
        vbs_path.push("run-silent.vbs");

        if vbs_path.exists() {
            std::fs::remove_file(&vbs_path).context("Failed to delete VBScript file")?;
        }

        Ok(())
    }

    pub fn get_vbs_path() -> Result<std::path::PathBuf> {
        let app_data =
            std::env::var("APPDATA").context("APPDATA environment variable not found")?;
        let mut vbs_path = std::path::PathBuf::from(app_data);
        vbs_path.push("mic-volume-control");
        vbs_path.push("run-silent.vbs");
        Ok(vbs_path)
    }
}
