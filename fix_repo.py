with open('crates/safe-core-persistence/src/repository.rs', 'r') as f:
    repo_content = f.read()

import re
repo_content = re.sub(r'pub async fn count_rules\(&self\) -> Result<usize, RepositoryError> \{\s*Ok\(0\)\s*\}', '''pub async fn count_rules(&self) -> Result<usize, RepositoryError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ethics_rules")
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }''', repo_content)

with open('crates/safe-core-persistence/src/repository.rs', 'w') as f:
    f.write(repo_content)
