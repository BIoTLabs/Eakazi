pub mod certificate;

use candid::{CandidType, Deserialize, Principal,candid_method};
use ic_cdk::api::call::ManualReply;
use ic_cdk::api::management_canister::main::raw_rand;
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use certificate::is_canister_custodian;
type IdStore = BTreeMap<String, Principal>;
type ProfileStore = BTreeMap<Principal, Profile>;
type CourseStore = BTreeMap<String, Course>;
type JobStore = BTreeMap<Principal, Jobs>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Profile {
    pub id: String,
    pub fullname: String,
    pub email: String,
    pub occupation: String,
    pub organization: String,
    pub location: String,
    pub resume: Vec<u8>,
    pub role: Roles,
    pub description: String,
    pub keywords: Vec<String>,
    pub skills: Vec<String>,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Course {
    pub id: String,
    pub title: String,
    pub creator: Principal,
    pub applicants: Vec<Principal>,
}
impl Default for Course {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            creator: ic_cdk::api::caller(),
            applicants: Default::default(),
        }
    }
}
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Jobs {
    pub id: String,
    pub title: String,
    pub creator: Principal,
    pub applicants: Vec<Principal>,
}

thread_local! {
    static PROFILE_STORE: RefCell<ProfileStore> = RefCell::default();
    static ID_STORE: RefCell<IdStore> = RefCell::default();
    static COURSE_STORE : RefCell<CourseStore> = RefCell::default();
    static JOB_STORE : RefCell<JobStore> = RefCell::default();
}

#[update]
async fn createUser(fullname: String, email: String, role: String) -> Profile {
    let principal_id = ic_cdk::api::caller();

    let uid = raw_rand().await.unwrap().0;
    let uid = String::from_utf8(uid).unwrap();
    let f = uid.clone();
    let id = uid.clone();
    let m = fullname.clone();
    let e = email.clone();
    ID_STORE.with(|el| el.borrow_mut().insert(uid, principal_id));
    PROFILE_STORE.with(|el| {
        el.borrow_mut().insert(
            principal_id,
            Profile {
                id: f,
                fullname,
                email,
                role: Roles::from_str(&role),
                ..Default::default()
            },
        )
    });

    Profile {
        id,
        fullname: m,
        email: e,
        role: Roles::from_str(&role),
        ..Default::default()
    }
}
#[query(name = "getSelf")]
fn get_self() -> Profile {
    let id = ic_cdk::api::caller();
    PROFILE_STORE.with(|profile_store| profile_store.borrow().get(&id).cloned().unwrap_or_default())
}

#[query]
fn get(uid: String) -> Profile {
    ID_STORE.with(|id_store| {
        PROFILE_STORE.with(|profile_store| {
            id_store
                .borrow()
                .get(&uid)
                .and_then(|id| profile_store.borrow().get(id).cloned())
                .unwrap_or_default()
        })
    })
}

#[update]
fn update(mut profile: Profile) {
    let principal_id = ic_cdk::api::caller();

    PROFILE_STORE.with(|profile_store| {
        profile_store
            .borrow_mut()
            .entry(principal_id)
            .and_modify(|mut el| {
                *el = Profile { ..profile };
            });
    });
}

#[query(manual_reply = true)]
fn search(text: String) -> ManualReply<Option<Profile>> {
    let text = text.to_lowercase();
    PROFILE_STORE.with(|profile_store| {
        for (_, p) in profile_store.borrow().iter() {
            if p.fullname.to_lowercase().contains(&text)
                || p.description.to_lowercase().contains(&text)
            {
                return ManualReply::one(Some(p));
            }

            for x in p.keywords.iter() {
                if x.to_lowercase() == text {
                    return ManualReply::one(Some(p));
                }
            }
        }
        ManualReply::one(None::<Profile>)
    })
}

#[update]
async fn createCourse(title: String) -> Course {
    let principal_id = ic_cdk::api::caller();
    let mut m: Profile = Default::default();
    PROFILE_STORE.with(|el| m = el.borrow().get(&principal_id).unwrap().clone());
    assert!(m.role == Roles::TRAINER);
    let uid = raw_rand().await.unwrap().0;
    let uid = String::from_utf8(uid).unwrap();
    let c = Course {
        id: uid,
        title: title.to_string(),
        creator: principal_id,
        applicants: vec![],
    };
    let d = c.clone();
    COURSE_STORE
        .with(|el| el.borrow_mut().insert(c.id, d))
        .unwrap()
    // COURSE_STORE.with_borrow_mut(|el| el.insert(c.id, d)).unwrap()
}

#[query]
fn getCourse(id: String) -> Course {
    COURSE_STORE.with(|el| el.borrow().get(&id).cloned().unwrap())
}
#[query]
fn getAllCourse() -> BTreeMap<String, Course> {
    COURSE_STORE.with(|el| el.borrow().clone())
}
#[update]
fn applyCourse(id: String) {
    let principal_id = ic_cdk::api::caller();
    COURSE_STORE.with(|el| {
      let _ =  el.borrow_mut()
            .clone()
            .entry(id)
            .and_modify(move |e| e.applicants.push(principal_id));
    });
    // COURSE_STORE.with_borrow_mut(|el| {
    // let _ =    el.clone().entry(id).and_modify(move |e|{
    //         e.applicants.push(principal_id)
    //     });
    // });
}
#[update]
fn applyJobs(id: Principal) {
    let principal_id = ic_cdk::api::caller();
    JOB_STORE.with(|el| {
        let _ = el.borrow_mut()
            .clone()
            .entry(id)
            .and_modify(move |e| e.applicants.push(principal_id));
    });
}

#[update]
async fn createJob(title: String) -> Jobs {
    let principal_id = ic_cdk::api::caller();
    let uid = raw_rand().await.unwrap().0;
    let uid = String::from_utf8(uid).unwrap();
    JOB_STORE.with(|el| {
        el.borrow_mut()
        .insert(
            principal_id,
            Jobs {
                id: uid,
                title,
                creator: principal_id,
                applicants: vec![],
            },
        )
        .unwrap()
    })
}

#[query]
fn getAllJobs() -> JobStore {
    JOB_STORE.with(|el| el.borrow().clone())
}
#[derive(CandidType, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum Roles {
    TRAINER,
    #[default]
    TRAINEE,
    EMPLOYER,
    ADMIN,
}
impl Roles {
    pub fn from_str(el: &str) -> Roles {
        match el {
            "Trainer" => Roles::TRAINER,
            "Trainee" => Roles::TRAINEE,
            "Employer" => Roles::EMPLOYER,
            _ => Roles::ADMIN,
        }
    }
}


ic_cdk::export_candid!();