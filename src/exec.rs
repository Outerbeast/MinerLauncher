use std::io;
use std::process::{Child, Command, ExitStatus};

pub enum MinerStatus
{
    Idle,
    Starting,
    Running,
    Stopped,
    Error(String),
}

pub struct MinerState
{
    pub child: Option<Child>,
    pub last_status: Option<ExitStatus>,
    pub status: MinerStatus,
}

impl Default for MinerState
{
    fn default() -> Self { Self::new() }
}
impl MinerState
{
    pub fn new() -> Self
    {
        Self
        {
            child: None,
            last_status: None,
            status: MinerStatus::Idle,
        }
    }

    pub fn launch(&mut self, exe: &str, args: &[&str], as_admin: bool) -> io::Result<()>
    {
        let args: Vec<&str> =
        args
            .iter()
            .copied()
            .filter( |a| *a != exe )
        .collect();

        self.status = MinerStatus::Starting;

        let process =
        match as_admin
        {
            true =>
            {// ISSUE: The child process in this case is NOT the miner but the terminal that runs executing the miner
                Command::new("powershell")
                    .arg( "-Command" )
                    .arg( format!( "Start-Process '{}' -Verb RunAs -ArgumentList '{}'", exe, args.join( " " ) ) )
                .spawn()?
            }

            false => Command::new( exe ).args( &args ).spawn()?

        };

        self.child = Some( process );
        self.status = MinerStatus::Running;

        println!( "Miner launched with args: {:?}", args );

        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<()>
    {
        if let Some( child ) = self.child.as_mut()
        {   // Best-effort kill
            let _ = child.kill();
            let status = child.wait()?;
            self.last_status = Some( status );
        }

        self.child = None;
        self.status = MinerStatus::Stopped;

        Ok(())
    }

    pub fn is_running(&mut self) -> bool
    {
        if let Some(child) = self.child.as_mut()
        {
            match child.try_wait()
            {
                Ok( Some( status ) ) =>
                {
                    self.last_status = Some(status);
                    self.status = MinerStatus::Stopped;
                    false
                }

                Ok( None ) => true,
                Err( _ ) => false
            }
        }
        else
        {
            false
        }
    }

    pub fn crashed(&self) -> bool
    {
        self.last_status
            .map(| s| !s.success() )
        .unwrap_or( false )
    }

    pub fn pid(&self) -> Option<u32>
    {
        self.child.as_ref().map( |c| c.id() )
    }
}
