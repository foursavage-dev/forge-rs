use std::sync::Arc;

pub struct JobserverPool {
    client: Arc<jobserver::Client>,
}

impl JobserverPool {
    pub fn new(limit: usize) -> std::io::Result<Self> {
        let client = jobserver::Client::new(limit)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub fn acquire(&self) -> std::io::Result<jobserver::Acquired> {
        self.client.acquire()
    }

    pub fn configure_command(&self, cmd: &mut std::process::Command) {
        self.client.configure(cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jobserver_pool_lifecycle() {
        let pool = JobserverPool::new(4).unwrap();
        let token1 = pool.acquire().unwrap();
        let token2 = pool.acquire().unwrap();
        drop(token1);
        drop(token2);
    }
}
