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

            let exe_path_str = exe_path
                .to_str()
                .context("Failed to convert executable path to string")?;
            exec_action
                .SetPath(&BSTR::from(exe_path_str))
                .context("Failed to set executable path")?;

            // Set arguments to call volume command in quiet mode
            let volume_percent = (target_volume * 100.0) as u8;
            let args = format!("--quiet volume {}", volume_percent);
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

            Ok(())
        }
    }

    pub fn is_registered(&self) -> bool {
        unsafe {
            match self.service.GetFolder(&BSTR::from(TASK_FOLDER)) {
                Ok(folder) => folder.GetTask(&BSTR::from(TASK_NAME)).is_ok(),
                Err(_) => false,
            }
        }
    }
}
