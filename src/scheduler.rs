use anyhow::{Context, Result};
use log::{debug, info};
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
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .context("Failed to initialize COM")?;

            let service: ITaskService =
                CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
                    .context("Failed to create TaskScheduler instance")?;

            service
                .Connect(None, None, None, None)
                .context("Failed to connect to Task Scheduler service")?;

            Ok(Self { service })
        }
    }

    pub fn register_autostart(&self) -> Result<()> {
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

        info!("Registering autostart task for: {}", exe_path.display());

        unsafe {
            // Get the root folder
            let root_folder = self
                .service
                .GetFolder(&BSTR::from(TASK_FOLDER))
                .context("Failed to get task folder")?;

            // Try to delete existing task if it exists
            match root_folder.DeleteTask(&BSTR::from(TASK_NAME), 0) {
                Ok(_) => info!("Removed existing task"),
                Err(_) => debug!("No existing task to remove"),
            }

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
                    "Maintains microphone volume at configured level",
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

            // Create trigger (at logon)
            let triggers = task_definition
                .Triggers()
                .context("Failed to get triggers collection")?;

            let trigger = triggers
                .Create(TASK_TRIGGER_LOGON)
                .context("Failed to create logon trigger")?;

            trigger
                .SetEnabled(VARIANT_TRUE)
                .context("Failed to enable trigger")?;

            // Set delay to prevent startup conflicts
            let logon_trigger: ILogonTrigger =
                trigger.cast().context("Failed to cast to ILogonTrigger")?;
            logon_trigger
                .SetDelay(&BSTR::from("PT30S"))
                .context("Failed to set delay")?;

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
                .SetExecutionTimeLimit(&BSTR::from("PT0S"))
                .context("Failed to set execution time limit")?;

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

            info!("Autostart task registered successfully");
            Ok(())
        }
    }

    pub fn unregister_autostart(&self) -> Result<()> {
        unsafe {
            let root_folder = self
                .service
                .GetFolder(&BSTR::from(TASK_FOLDER))
                .context("Failed to get task folder")?;

            root_folder
                .DeleteTask(&BSTR::from(TASK_NAME), 0)
                .context("Failed to delete task")?;

            info!("Autostart task unregistered successfully");
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

impl Drop for TaskScheduler {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
