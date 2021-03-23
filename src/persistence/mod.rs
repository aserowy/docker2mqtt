use crate::configuration::Configuration;

pub trait Repository {

}

struct NoPersistenceRepository {

}

impl Repository for NoPersistenceRepository {

}

struct SledRepository {

}

impl Repository for SledRepository {

}

pub async fn spin_up(conf: &Configuration) {
    let repository = create_repository(conf);
}

fn create_repository(conf: &Configuration) -> Box<dyn Repository> {
    if conf.persistence.is_some() {
        Box::new(SledRepository {

        })
    } else {
        Box::new(NoPersistenceRepository {

        })
    }
}