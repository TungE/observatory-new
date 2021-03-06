use crate::models::*;
use diesel::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use ring::rand::{SecureRandom, SystemRandom};
use ring::{digest, pbkdf2};

//# Attendance Code Helpers

pub fn verify_code(conn: &SqliteConnection, vcode: &String) -> Option<Box<dyn Attendable>> {
    if let Some(e) = {
        use crate::schema::events::dsl::*;
        events
            .filter(code.eq(vcode))
            .first::<Event>(conn)
            .optional()
            .expect("Failed to get events from database")
    } {
        Some(Box::new(e))
    } else {
        if let Some(m) = {
            use crate::schema::meetings::dsl::*;
            meetings
                .filter(code.eq(vcode))
                .first::<Meeting>(conn)
                .optional()
                .expect("Failed to get meetings from database")
        } {
            Some(Box::new(m))
        } else {
            None
        }
    }
}

pub fn attendance_code(conn: &SqliteConnection) -> String {
    let code = gen_code();
    if verify_code(conn, &code).is_some() {
        attendance_code(conn)
    } else {
        code
    }
}

pub fn gen_code() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .collect::<String>()
        .to_lowercase()
}

//# Filter Helpers

pub fn filter_users(conn: &SqliteConnection, term: Option<String>) -> Vec<User> {
    use crate::schema::users::dsl::*;

    if let Some(term) = term {
        let sterm = format!("%{}%", term);
        let filter = real_name
            .like(&sterm)
            .or(email.like(&sterm))
            .or(handle.like(&sterm));
        users.filter(filter).load(conn)
    } else {
        users.load(conn)
    }
    .expect("Failed to get users")
}

pub fn filter_projects(conn: &SqliteConnection, term: Option<String>) -> Vec<Project> {
    use crate::schema::projects::dsl::*;

    if let Some(term) = term {
        let sterm = format!("%{}%", term);
        let filter = name.like(&sterm);
        projects.filter(filter).load(conn)
    } else {
        projects.load(conn)
    }
    .expect("Failed to get projects")
}

//# Password Helpers

const N_ITER: u32 = 100000;
const CRE_LEN: usize = digest::SHA512_256_OUTPUT_LEN;

pub fn gen_salt() -> String {
    let rng = SystemRandom::new();
    let mut salt = [0u8; CRE_LEN];
    rng.fill(&mut salt).unwrap();
    unsafe { String::from_utf8_unchecked(salt.to_vec()) }
}

pub fn hash_password(pass: String, salt: &String) -> String {
    let mut out = [0u8; CRE_LEN];
    pbkdf2::derive(
        &digest::SHA512,
        N_ITER,
        salt.as_bytes(),
        pass.as_bytes(),
        &mut out,
    );
    unsafe { String::from_utf8_unchecked(out.to_vec()) }
}

pub fn verify_password(pass: String, compare_to: String, salt: &String) -> bool {
    pbkdf2::verify(
        &digest::SHA512,
        N_ITER,
        salt.as_bytes(),
        pass.as_bytes(),
        compare_to.as_bytes(),
    )
    .is_ok()
}
