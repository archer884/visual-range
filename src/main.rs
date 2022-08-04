use std::{self, process};

use clap::Parser;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("observer must have positive non-zero height: {0}")]
    Observer(f64),
    #[error("subject must have positive non-zero height: {0}")]
    Subject(f64),
}

#[derive(Clone, Debug, Parser)]
struct Args {
    /// height of the observer
    observer: f64,

    /// height of the subject
    ///
    /// Default: 0 ("the horizon")
    subject: Option<f64>,

    /// calculate using height in meters
    #[clap(short, long)]
    metric: bool,
}

impl Args {
    fn validate(&self) -> Result<()> {
        if self.observer <= 0.0 {
            return Err(Error::Observer(self.observer));
        }

        if let Some(subject) = self.subject {
            if subject <= 0.0 {
                return Err(Error::Subject(subject));
            }
        }

        Ok(())
    }
}

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    /// meters per foot
    const CONVERSION_FACTOR_FEET_TO_METERS: f64 = 1.0 / 3.28084;

    /// miles per meter
    ///
    /// Again, blame Google.
    const CONVERSION_FACTOR_METERS_TO_MILES: f64 = 0.00062137 / 1.0;

    /// km per meter
    const CONVERSION_FACTOR_METERS_TO_KM: f64 = 1.0 / 1000.0;

    /// earth radius in meters (average value)
    ///
    /// Taken from WolframAlpha.
    const EARTH_RADIUS: f64 = 6371009.0;

    args.validate()?;

    // My theory is that the distance across the earth's surface covered by an observer at a
    // given height is equal to the distance described by an angle having the same measurement
    // as the ... skinny point of a triangle constructed to match the observer's viewpoint. I'd
    // ascii that up for you, but I'm not an artist.

    // Unless the user has specified metric measurements, we assume the height has been given in
    // feet. This is because the most realistic real-world use case for this involves aircraft at
    // a given altitude, and aircraft altitude is always given in feet. (Except maybe in China and
    // in Soviet Russia, but screw them.)

    let effective_observer_height = if args.metric {
        args.observer + args.subject.unwrap_or_default() + EARTH_RADIUS
    } else {
        let height = args.observer + args.subject.unwrap_or_default();
        height * CONVERSION_FACTOR_FEET_TO_METERS + EARTH_RADIUS
    };

    // The effective observer height is the height of the observer plus the height of the subject.
    // This represents the hypotenuse of the triangle we are using to figure out the angle for the
    // solution to our problem.

    // For those of you who like geometry as much as me, the 'hypotenuse' is the long leg of a
    // right triangle.

    // We're actually looking for angle α of a right triangle, and the formula for that is just
    // arccos(b / c), where c is the hypotenuse and b is the opposite side of the angle. In this
    // case, c is the effective height of the observer, while b is the radius of the earth itself.

    let alpha = (EARTH_RADIUS / effective_observer_height).acos();
    let slant_range_meters = ((effective_observer_height * effective_observer_height)
        - (EARTH_RADIUS * EARTH_RADIUS))
        .sqrt();

    // "Slant range," calculated above, is commonly the diagonal rather than horizontal distance
    // from an airborn observer to a land-based subject. For instance, it is the distance a bullet
    // must travel from a strafing aircraft to the target. It's not essential for our purposes,
    // but the calculation is basically free at this point, so why not?

    // Now, to calculate the ground distance covered by the observer, we consider the angle alpha
    // (α) as instead theta (θ), being the central angle of the arc. We then multiply this by a
    // new "alpha," this being the radius of the circle. I hate mathematicians. Why can't you guys
    // learn to name your variables properly? Once more, the radius in question is merely the
    // average radius of earth.

    let ground_distance_meters = alpha * EARTH_RADIUS;
    let ground_distance_km = ground_distance_meters * CONVERSION_FACTOR_METERS_TO_KM;
    let ground_distance_miles = ground_distance_meters * CONVERSION_FACTOR_METERS_TO_MILES;

    println!("Ground distance:\n\n{ground_distance_meters:.00} m\n{ground_distance_miles:.02} mi\n{ground_distance_km:.02} km\n\nSlant range:\n\n{slant_range_meters:.00} m");

    Ok(())
}
