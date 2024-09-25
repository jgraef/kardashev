use nalgebra::{
    Point3,
    Vector3,
};
use palette::{
    LinSrgb,
    LinSrgba,
};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    postgres::{
        PgArgumentBuffer,
        PgTypeInfo,
        PgValueRef,
    },
    Decode,
    Encode,
    Postgres,
};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Vec3(pub nalgebra::Vector3<f32>);

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(nalgebra::Vector3::new(x, y, z))
    }
}

impl From<Point3<f32>> for Vec3 {
    fn from(value: Point3<f32>) -> Self {
        value.coords.into()
    }
}

impl From<Vec3> for Point3<f32> {
    fn from(value: Vec3) -> Self {
        Point3::from(Vector3::from(value))
    }
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "vec3")]
struct Vec3Adapter {
    x: f32,
    y: f32,
    z: f32,
}

impl sqlx::Type<Postgres> for Vec3 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("vec3")
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for Vec3 {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        Encode::<'q, Postgres>::encode(
            Vec3Adapter {
                x: self.0.x,
                y: self.0.y,
                z: self.0.z,
            },
            buf,
        )
    }
}

impl<'r> Decode<'r, Postgres> for Vec3 {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let adapter = <Vec3Adapter as Decode<'r, Postgres>>::decode(value)?;
        Ok(Self(Vector3::new(adapter.x, adapter.y, adapter.z)))
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Rgba(pub palette::LinSrgba);

impl Rgba {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(palette::LinSrgba::new(red, green, blue, alpha))
    }
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "rgba")]
struct RgbaAdapter {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl sqlx::Type<Postgres> for Rgba {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("rgba")
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for Rgba {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        Encode::<'q, Postgres>::encode(
            RgbaAdapter {
                r: self.0.color.red,
                g: self.0.color.green,
                b: self.0.color.blue,
                a: self.0.alpha,
            },
            buf,
        )
    }
}

impl<'r> Decode<'r, Postgres> for Rgba {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let adapter = <RgbaAdapter as Decode<'r, Postgres>>::decode(value)?;
        Ok(Self(LinSrgba::new(
            adapter.r, adapter.g, adapter.b, adapter.a,
        )))
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Rgb(pub palette::LinSrgb);

impl Rgb {
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self(palette::LinSrgb::new(red, green, blue))
    }
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "rgb")]
struct RgbAdapter {
    r: f32,
    g: f32,
    b: f32,
}

impl sqlx::Type<Postgres> for Rgb {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("rgb")
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for Rgb {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        Encode::<'q, Postgres>::encode(
            RgbAdapter {
                r: self.0.red,
                g: self.0.green,
                b: self.0.blue,
            },
            buf,
        )
    }
}

impl<'r> Decode<'r, Postgres> for Rgb {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let adapter = <RgbAdapter as Decode<'r, Postgres>>::decode(value)?;
        Ok(Self(LinSrgb::new(adapter.r, adapter.g, adapter.b)))
    }
}
