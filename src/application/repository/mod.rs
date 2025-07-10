pub mod movie_repo;
pub mod user_repo;

pub type RepositoryResult<T> = Result<T, sqlx::Error>;
