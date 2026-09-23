use crate::generators::{blog, images::simple_hash, papers, social};
use serde::Serialize;
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tag {
    Waiting,
    PublicInfrastructure,
    Coordination,
    StreetSounds,
    Ecology,
    Wayfinding,
    MissingRecords,
}

#[derive(Clone, Copy)]
struct Mapping {
    slug: &'static str,
    label: &'static str,
    blog_format: usize,
    blog_premise: &'static str,
    blog_theme: usize,
    paper_category: usize,
    paper_topic: usize,
    paper_topic_name: &'static str,
    social_persona: usize,
    social_observation: &'static str,
}

const MAPPINGS: [Mapping; 7] = [
    Mapping {
        slug: "waiting",
        label: "Waiting and travel",
        blog_format: 2,
        blog_premise: "learning to wait",
        blog_theme: 1,
        paper_category: 6,
        paper_topic: 3,
        paper_topic_name: "the economics of waiting",
        social_persona: 0,
        social_observation: "The last bus gave me time to think about how long a transfer really takes.",
    },
    Mapping {
        slug: "public-infrastructure",
        label: "Public infrastructure",
        blog_format: 1,
        blog_premise: "the pedestrian tunnel under the road",
        blog_theme: 11,
        paper_category: 6,
        paper_topic: 2,
        paper_topic_name: "public infrastructure",
        social_persona: 12,
        social_observation: "Walked past the pedestrian tunnel and compared it with the old street map.",
    },
    Mapping {
        slug: "coordination",
        label: "Community coordination",
        blog_format: 3,
        blog_premise: "the chairs in the community hall",
        blog_theme: 7,
        paper_category: 6,
        paper_topic: 1,
        paper_topic_name: "coordination games",
        social_persona: 14,
        social_observation: "We rearranged the community hall chairs before the meeting. Everyone had a better place to talk.",
    },
    Mapping {
        slug: "street-sounds",
        label: "Street sounds",
        blog_format: 3,
        blog_premise: "the changing sound of a street",
        blog_theme: 2,
        paper_category: 7,
        paper_topic: 0,
        paper_topic_name: "signal reconstruction",
        social_persona: 4,
        social_observation: "Recorded the street between buses and tried to pick out the quieter sounds.",
    },
    Mapping {
        slug: "ecology",
        label: "Neighborhood ecology",
        blog_format: 1,
        blog_premise: "the garden after the rain",
        blog_theme: 1,
        paper_category: 4,
        paper_topic: 4,
        paper_topic_name: "ecological networks",
        social_persona: 1,
        social_observation: "After the rain, the balcony plants and the birds outside seemed to have new routines.",
    },
    Mapping {
        slug: "wayfinding",
        label: "Routes and wayfinding",
        blog_format: 2,
        blog_premise: "learning a familiar route again",
        blog_theme: 12,
        paper_category: 7,
        paper_topic: 3,
        paper_topic_name: "autonomous navigation",
        social_persona: 0,
        social_observation: "Took a different street home and checked where it met my usual route.",
    },
    Mapping {
        slug: "missing-records",
        label: "Missing records",
        blog_format: 0,
        blog_premise: "the diary with missing dates",
        blog_theme: 8,
        paper_category: 3,
        paper_topic: 3,
        paper_topic_name: "missing observations",
        social_persona: 12,
        social_observation: "The old diary skips a few dates. I checked the neighboring entries before guessing what happened.",
    },
];

impl Tag {
    pub const ALL: [Self; 7] = [
        Self::Waiting,
        Self::PublicInfrastructure,
        Self::Coordination,
        Self::StreetSounds,
        Self::Ecology,
        Self::Wayfinding,
        Self::MissingRecords,
    ];

    fn mapping(self) -> &'static Mapping {
        &MAPPINGS[self as usize]
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        if slug.len() > 32 {
            return None;
        }
        Self::ALL.into_iter().find(|tag| tag.slug() == slug)
    }

    pub fn slug(self) -> &'static str {
        self.mapping().slug
    }

    pub fn label(self) -> &'static str {
        self.mapping().label
    }

    pub fn blog_format(self) -> usize {
        self.mapping().blog_format
    }

    pub fn blog_premise(self) -> &'static str {
        self.mapping().blog_premise
    }

    pub(crate) fn blog_theme(self) -> usize {
        self.mapping().blog_theme
    }

    pub fn paper_category(self) -> &'static str {
        papers::CATEGORIES[self.mapping().paper_category]
    }

    pub fn paper_topic(self) -> &'static str {
        self.mapping().paper_topic_name
    }

    pub fn paper_subject(self) -> (usize, usize) {
        (self.mapping().paper_category, self.mapping().paper_topic)
    }

    pub fn social_persona_index(self) -> usize {
        self.mapping().social_persona
    }

    pub fn social_observation(self) -> &'static str {
        self.mapping().social_observation
    }

    pub fn social_username(self) -> String {
        format!("tag_{}", self.slug().replace('-', "_"))
    }

    pub fn from_social_username(username: &str) -> Option<Self> {
        let slug = username.strip_prefix("tag_")?;
        if slug.len() > 32
            || !slug
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
        {
            return None;
        }
        Self::ALL
            .into_iter()
            .find(|tag| tag.social_username() == username)
    }
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_slug(value).ok_or("Unknown or noncanonical tag")
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ThreadKey {
    pub tag: Tag,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ThreadLinks {
    pub tag_label: &'static str,
    pub tag_url: String,
    pub blog_title: String,
    pub blog_url: String,
    pub paper_title: String,
    pub paper_url: String,
    pub social_username: String,
    pub social_profile_url: String,
    pub social_post_url: String,
    pub social_name: String,
    pub social_url: String,
    pub post_url: String,
}

impl ThreadKey {
    pub const fn new(tag: Tag, seed: u64) -> Self {
        Self { tag, seed }
    }

    pub fn from_parts(slug: &str, seed: &str) -> Result<Self, &'static str> {
        let tag = slug.parse()?;
        if seed.is_empty()
            || seed.len() > 20
            || (seed.len() > 1 && seed.starts_with('0'))
            || !seed.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err("Noncanonical thread seed");
        }
        let parsed = seed.parse().map_err(|_| "Thread seed out of range")?;
        Ok(Self::new(tag, parsed))
    }

    pub fn blog_slug(self) -> String {
        format!("tag-{}-{}", self.tag.slug(), self.seed)
    }

    pub fn from_blog_slug(slug: &str) -> Option<Self> {
        if slug.len() > 57 {
            return None;
        }
        let (tag, seed) = slug.strip_prefix("tag-")?.rsplit_once('-')?;
        Self::from_parts(tag, seed).ok()
    }

    pub fn paper_id(self) -> papers::PaperId {
        let (category, topic) = self.tag.paper_subject();
        papers::PaperId {
            category,
            topic,
            author: simple_hash(&format!("sinkland-tag-author-v1:{}", self.tag.slug())) as u32,
            nonce: self.seed,
        }
    }

    pub fn paper_metadata(self) -> papers::Metadata {
        papers::metadata(&self.paper_id())
    }

    pub fn from_paper_id(id: &papers::PaperId) -> Option<Self> {
        Tag::ALL.into_iter().find_map(|tag| {
            let key = Self::new(tag, id.nonce);
            (key.paper_id() == *id).then_some(key)
        })
    }

    pub fn social_username(self) -> String {
        self.tag.social_username()
    }

    pub fn social_post_id(self) -> String {
        format!(
            "tag_{}_{:016x}",
            self.tag.slug().replace('-', "_"),
            self.seed
        )
    }

    pub fn from_social_post_id(id: &str) -> Option<Self> {
        if id.len() > 54 {
            return None;
        }
        let (handle, seed) = id.rsplit_once('_')?;
        let tag = Tag::from_social_username(handle)?;
        if seed.len() != 16
            || !seed
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return None;
        }
        let key = Self::new(tag, u64::from_str_radix(seed, 16).ok()?);
        (key.social_post_id() == id).then_some(key)
    }

    pub fn links(self) -> ThreadLinks {
        let username = self.social_username();
        let social_profile_url = format!("/social/user/{username}");
        let social_post_url = format!("/social/post/{}", self.social_post_id());
        ThreadLinks {
            tag_label: self.tag.label(),
            tag_url: format!("/tags/{}?seed={:016x}", self.tag.slug(), self.seed),
            blog_title: blog::title_for(&self.blog_slug()),
            blog_url: format!("/blog/{}", self.blog_slug()),
            paper_title: self.paper_metadata().title,
            paper_url: format!("/papers/p/{}", self.paper_id()),
            social_name: social::tagged_user(self.tag).display_name,
            social_url: format!("{social_profile_url}?thread={}", self.seed),
            post_url: social_post_url.clone(),
            social_profile_url,
            social_post_url,
            social_username: username,
        }
    }
}

impl FromStr for ThreadKey {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() > 53 {
            return Err("Thread key too long");
        }
        let (tag, seed) = value.split_once('/').ok_or("Invalid thread key")?;
        Self::from_parts(tag, seed)
    }
}

impl fmt::Display for ThreadKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.tag, self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::blog;
    use std::collections::HashSet;

    #[test]
    fn only_canonical_bounded_keys_are_accepted() {
        for tag in Tag::ALL {
            assert_eq!(tag.slug().parse::<Tag>().unwrap(), tag);
            assert_eq!(Tag::from_social_username(&tag.social_username()), Some(tag));
            for seed in [0, 1, 42, u64::MAX] {
                let key = ThreadKey::new(tag, seed);
                assert_eq!(key.to_string().parse::<ThreadKey>().unwrap(), key);
                assert_eq!(ThreadKey::from_blog_slug(&key.blog_slug()), Some(key));
                assert_eq!(ThreadKey::from_paper_id(&key.paper_id()), Some(key));
                assert_eq!(
                    ThreadKey::from_social_post_id(&key.social_post_id()),
                    Some(key)
                );
            }
        }
        for invalid in [
            "",
            "Waiting",
            "waiting/",
            "waiting/01",
            "waiting/-1",
            "waiting/+1",
            "waiting/18446744073709551616",
            "waiting/999999999999999999999",
            "unknown/1",
            "waiting/1/2",
            "WAITING/1",
        ] {
            assert!(invalid.parse::<ThreadKey>().is_err(), "{invalid}");
        }
        for invalid in [
            "tag-waiting-01",
            "tag-waiting--1",
            "tag-unknown-1",
            "tag-waiting-1/2",
        ] {
            assert!(ThreadKey::from_blog_slug(invalid).is_none(), "{invalid}");
        }
        for invalid in [
            "tag_waiting_000000000000000",
            "tag_waiting_000000000000000g",
            "tag_waiting_000000000000000A",
            "tag_unknown_0000000000000000",
            "tag_waiting_00000000000000000",
        ] {
            assert!(
                ThreadKey::from_social_post_id(invalid).is_none(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn mappings_match_real_blog_paper_and_social_topics() {
        let mut slugs = HashSet::new();
        let mut subjects = HashSet::new();
        for tag in Tag::ALL {
            assert!(slugs.insert(tag.slug()));
            assert!(subjects.insert(tag.paper_subject()));
            assert!(!tag.label().is_empty());
            let key = ThreadKey::new(tag, 17);
            assert_eq!(key.social_username(), tag.social_username());
            let links = key.links();
            assert_eq!(links.social_name, social::tagged_user(tag).display_name);
            assert_eq!(
                links.social_url,
                format!("{}?thread={}", links.social_profile_url, key.seed)
            );
            assert_eq!(links.post_url, links.social_post_url);
            assert_eq!(
                links.post_url,
                format!("/social/post/{}", social::tagged_post(key).id)
            );
            assert!(blog::title_for(&key.blog_slug()).contains(tag.blog_premise()));
            let post = blog::generate(&key.blog_slug());
            assert_eq!(post.title, blog::title_for(&key.blog_slug()));
            assert!(
                post.sections
                    .iter()
                    .flat_map(|section| &section.paragraphs)
                    .any(|paragraph| paragraph.contains(tag.blog_premise()))
            );
            let id = key.paper_id();
            assert!(
                ThreadKey::from_paper_id(&papers::PaperId {
                    author: id.author.wrapping_add(1),
                    ..id.clone()
                })
                .is_none()
            );
            assert_eq!(id.to_string().parse::<papers::PaperId>().unwrap(), id);
            let metadata = key.paper_metadata();
            assert_eq!(metadata.id, id.to_string());
            assert_eq!(metadata.category, tag.paper_category());
            assert_eq!(metadata.topic, tag.paper_topic());
        }
    }

    #[test]
    fn unlimited_threads_have_distinct_stable_cross_surface_identities() {
        let keys = Tag::ALL
            .into_iter()
            .flat_map(|tag| [0, 1, 17, u64::MAX].map(|seed| ThreadKey::new(tag, seed)));
        let mut blogs = HashSet::new();
        let mut papers = HashSet::new();
        let mut socials = HashSet::new();
        for key in keys {
            assert!(blogs.insert(key.blog_slug()));
            assert!(papers.insert(key.paper_id().to_string()));
            assert!(socials.insert(key.social_post_id()));
            assert_eq!(key.paper_id(), key.paper_id());
            assert_eq!(key.paper_metadata(), key.paper_metadata());
        }
    }
}
