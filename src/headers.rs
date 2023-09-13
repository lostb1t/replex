use http::HeaderName;
use salvo::http::headers::Header;

pub const PLEX_TOKEN: HeaderName = HeaderName::from_static("x-plex-token");
pub const PLEX_LANGUAGE: HeaderName = HeaderName::from_static("x-plex-language");
pub const PLEX_PLATFORM: HeaderName = HeaderName::from_static("x-plex-platform");
pub const PLEX_CLIENT_IDENTIFIER: HeaderName = HeaderName::from_static("x-plex-client-identifier");
pub const PLEX_CLIENT_PROFILE_EXTRA: HeaderName = HeaderName::from_static("x-plex-client-profile-extra");