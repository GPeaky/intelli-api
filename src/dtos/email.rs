use super::RegisterUserDto;

#[derive(Debug)]
pub struct EmailUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
}

impl<'a> From<&'a RegisterUserDto> for EmailUser<'a> {
    fn from(user: &'a RegisterUserDto) -> Self {
        Self {
            username: &user.username,
            email: &user.email,
        }
    }
}
