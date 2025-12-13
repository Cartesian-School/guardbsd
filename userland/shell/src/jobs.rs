//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: shell
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Kontrola zadań (inspirowana bash/zsh) w powłoce gsh.

use gbsd::*;

#[derive(Copy, Clone, PartialEq)]
pub enum JobState {
    Running,
    Stopped,
    Done,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Job {
    pub id: usize,
    pub pid: i32,
    pub state: JobState,
    pub command: [u8; 256],
    pub cmd_len: usize,
}

impl Job {
    pub const fn new() -> Self {
        Self {
            id: 0,
            pid: 0,
            state: JobState::Done,
            command: [0; 256],
            cmd_len: 0,
        }
    }
}

pub struct JobControl {
    jobs: [Option<Job>; 32],
    job_count: usize,
    next_job_id: usize,
    foreground_job: Option<usize>,
}

impl JobControl {
    pub const fn new() -> Self {
        Self {
            jobs: [None; 32],
            job_count: 0,
            next_job_id: 1,
            foreground_job: None,
        }
    }

    /// Add a new job (background or suspended)
    pub fn add_job(&mut self, pid: i32, command: &[u8], background: bool) -> usize {
        let job_id = self.next_job_id;
        self.next_job_id += 1;

        let mut job = Job::new();
        job.id = job_id;
        job.pid = pid;
        job.state = if background {
            JobState::Running
        } else {
            JobState::Stopped
        };

        let len = command.len().min(255);
        job.command[..len].copy_from_slice(&command[..len]);
        job.cmd_len = len;

        // Find empty slot
        for i in 0..32 {
            if self.jobs[i].is_none() {
                self.jobs[i] = Some(job);
                self.job_count += 1;

                if background {
                    let _ = crate::io::print(b"[");
                    let _ = JobControl::print_number(job_id);
                    let _ = crate::io::print(b"] ");
                    let _ = JobControl::print_number(pid as usize);
                    let _ = crate::io::println(b"");
                }

                return job_id;
            }
        }

        0 // No space
    }

    /// List all jobs
    pub fn list_jobs(&self) -> Result<()> {
        for job_opt in &self.jobs {
            if let Some(ref job) = job_opt {
                // [1]+ 1234 Running    command
                let _ = crate::io::print(b"[");
                let _ = JobControl::print_number(job.id);
                let _ = crate::io::print(b"]");

                // Mark current/previous job
                if self.foreground_job == Some(job.id) {
                    let _ = crate::io::print(b"+ ");
                } else {
                    let _ = crate::io::print(b"  ");
                }

                let _ = JobControl::print_number(job.pid as usize);
                let _ = crate::io::print(b" ");

                match job.state {
                    JobState::Running => {
                        let _ = crate::io::print(b"Running    ");
                    }
                    JobState::Stopped => {
                        let _ = crate::io::print(b"Stopped    ");
                    }
                    JobState::Done => {
                        let _ = crate::io::print(b"Done       ");
                    }
                }

                let _ = crate::io::println(&job.command[..job.cmd_len]);
            }
        }
        Ok(())
    }

    /// Bring job to foreground
    pub fn foreground(&mut self, job_id: usize) -> Result<()> {
        let pid;
        {
            let job = self.find_job_mut(job_id).ok_or(Error::NotFound)?;

            if job.state == JobState::Stopped {
                // Resume stopped job
                // TODO: Send SIGCONT to job.pid
                job.state = JobState::Running;
            }

            pid = job.pid;
        }

        self.foreground_job = Some(job_id);

        // Wait for job to complete
        if let Some((_pid, _status)) = gbsd::process::waitpid(pid, 0)? {
            // Status currently unused
        }

        // Mark job as done
        if let Some(job) = self.find_job_mut(job_id) {
            job.state = JobState::Done;
        }
        self.foreground_job = None;

        Ok(())
    }

    /// Send job to background
    pub fn background(&mut self, job_id: usize) -> Result<()> {
        let job = self.find_job_mut(job_id).ok_or(Error::NotFound)?;

        if job.state == JobState::Stopped {
            // Resume stopped job in background
            // TODO: Send SIGCONT to job.pid
            job.state = JobState::Running;

            let _ = crate::io::print(b"[");
            let _ = JobControl::print_number(job.id);
            let _ = crate::io::print(b"]+ ");
            let _ = crate::io::println(&job.command[..job.cmd_len]);
        }

        Ok(())
    }

    /// Check for completed jobs
    pub fn check_jobs(&mut self) {
        for i in 0..32 {
            if let Some(ref mut job) = self.jobs[i] {
                if job.state == JobState::Running {
                    // Non-blocking wait
                    if let Ok(Some((_pid, _status))) =
                        gbsd::process::waitpid(job.pid, gbsd::process::WNOHANG)
                    {
                        job.state = JobState::Done;

                        let _ = crate::io::print(b"[");
                        let _ = JobControl::print_number(job.id);
                        let _ = crate::io::print(b"]+ Done       ");
                        let _ = crate::io::println(&job.command[..job.cmd_len]);
                    }
                }
            }
        }

        // Clean up done jobs
        for i in 0..32 {
            if let Some(ref job) = self.jobs[i] {
                if job.state == JobState::Done {
                    self.jobs[i] = None;
                    self.job_count -= 1;
                }
            }
        }
    }

    fn find_job_mut(&mut self, job_id: usize) -> Option<&mut Job> {
        for job_opt in &mut self.jobs {
            if let Some(ref mut job) = job_opt {
                if job.id == job_id {
                    return Some(job);
                }
            }
        }
        None
    }

    fn print_number(mut num: usize) -> Result<()> {
        if num == 0 {
            return crate::io::print(b"0");
        }

        let mut buf = [0u8; 10];
        let mut pos = 0;

        while num > 0 {
            buf[pos] = b'0' + (num % 10) as u8;
            num /= 10;
            pos += 1;
        }

        // Print in reverse
        while pos > 0 {
            pos -= 1;
            let _ = crate::io::print(&[buf[pos]]);
        }

        Ok(())
    }
}
