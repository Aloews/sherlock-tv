use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DynastyId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub birth_year: i32,
    pub parent_ids: Vec<PersonId>,
    pub is_alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dynasty {
    pub id: DynastyId,
    pub founder_id: PersonId,
    pub members: BTreeMap<PersonId, Person>,
    pub succession_order: Vec<PersonId>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DynastyError {
    #[error("founder must be present in dynasty membership")]
    MissingFounder,
    #[error("person already exists")]
    DuplicatePerson,
    #[error("a parent must already be a member of this dynasty")]
    UnknownParent,
    #[error("succession order contains a duplicate person")]
    DuplicateSuccessor,
    #[error("succession order references a non-member")]
    UnknownSuccessor,
}

impl Dynasty {
    pub fn new(id: DynastyId, founder: Person) -> Self {
        let founder_id = founder.id;
        let mut members = BTreeMap::new();
        members.insert(founder_id, founder);
        Self {
            id,
            founder_id,
            members,
            succession_order: vec![founder_id],
        }
    }

    pub fn add_member(&mut self, person: Person) -> Result<(), DynastyError> {
        if self.members.contains_key(&person.id) {
            return Err(DynastyError::DuplicatePerson);
        }
        if person.parent_ids.iter().any(|parent| !self.members.contains_key(parent)) {
            return Err(DynastyError::UnknownParent);
        }
        self.members.insert(person.id, person);
        Ok(())
    }

    pub fn set_succession_order(&mut self, order: Vec<PersonId>) -> Result<(), DynastyError> {
        let mut seen = std::collections::BTreeSet::new();
        for person_id in &order {
            if !self.members.contains_key(person_id) {
                return Err(DynastyError::UnknownSuccessor);
            }
            if !seen.insert(*person_id) {
                return Err(DynastyError::DuplicateSuccessor);
            }
        }
        self.succession_order = order;
        Ok(())
    }

    pub fn successor(&self) -> Option<&Person> {
        self.succession_order
            .iter()
            .filter_map(|id| self.members.get(id))
            .find(|person| person.is_alive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(id: u64, name: &str, parents: Vec<PersonId>) -> Person {
        Person {
            id: PersonId(id),
            name: name.into(),
            birth_year: 1900 + id as i32,
            parent_ids: parents,
            is_alive: true,
        }
    }

    #[test]
    fn succession_skips_deceased_members() {
        let founder = person(1, "Founder", vec![]);
        let mut dynasty = Dynasty::new(DynastyId(1), founder);
        dynasty.members.get_mut(&PersonId(1)).expect("founder").is_alive = false;
        dynasty.add_member(person(2, "Heir", vec![PersonId(1)])).expect("add heir");
        dynasty.set_succession_order(vec![PersonId(1), PersonId(2)]).expect("order");

        assert_eq!(dynasty.successor().map(|person| person.id), Some(PersonId(2)));
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let mut dynasty = Dynasty::new(DynastyId(1), person(1, "Founder", vec![]));
        let result = dynasty.add_member(person(2, "Unknown child", vec![PersonId(99)]));

        assert_eq!(result, Err(DynastyError::UnknownParent));
    }
}
