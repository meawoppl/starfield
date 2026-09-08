# Resolved Solar-System Bodies — Implementation Plan

Covers issues #157–#170. All fourteen serve one consumer: `focalplane` is adding
resolved planets, the Moon and atmospheric limbs to its focal-plane renderer,
with a headline study of guiding on the Earth limb from Mars orbit. The split
agreed with focalplane is: **geometry lives in starfield, radiometry stays in
focalplane.** This document turns the fourteen issues into an ordered set of
pull requests, fixes the design questions the issues leave open, and records the
validation strategy.

## 1. What exists today

Every claim the issues make about the codebase was checked and holds:

| Area | State | Relevant to |
|---|---|---|
| `jplephem::pck` | 24-line stub; opens the DAF and nothing else | #158 |
| `jplephem::daf`, `jplephem::chebyshev` | Complete; SPK types 2, 3, 21 supported | #158 |
| `data::downloader::resolve_url` | Knows `.bsp` only | #169 |
| `framelib::Frame` | Trait with one method `rotation_at(&Time) -> Matrix3` (ICRF → frame); 5 impls | #159, #161 |
| `Time::c_matrix()` | **Full ICRF → ITRS rotation already exists**: `W · Rz(−GAST) · M`, polar motion applied when a table is loaded | #161 |
| `planetlib::Body` | Enum of 11 bodies with `name()` / `naif_id()`; no physical constants | #160 |
| `constants::EARTH_RADIUS` | The only radius in the crate | #160, #165 |
| `magnitudelib` | Private `angle_between`, Sun approximated at SSB, `301` → `UnsupportedBody` | #162, #168 |
| `positions::Position` | `observe(name, kernel, t)`, `apparent(kernel, t)`, `observer_barycentric: Option<Box<Position>>`; `apparent()` asserts `observer_barycentric` is set | #162–#167 |
| `keplerlib::KeplerOrbit::at(t)` | Heliocentric only | #167 |
| `horizons::parser` | Parses observer tables with arbitrary QUANTITIES | validation of #163–#165 |
| `searchlib::find_discrete` | Exists | #170 |
| `pybridge` | Embedded Python with skyfield 1.53, astropy, jplephem 2.22 — **no spiceypy** | validation |

## 2. Design decisions

These must be settled before the first PR, because three issues disagree with
each other about where things live.

### 2.1 New module `planetarylib`, a port of Skyfield's `planetarylib.py`

Issue #157 puts `RotationalElements` in `jplephem::pck`; #160 has
`planetlib::Body` return "the same type"; #159 consumes it from `framelib`.
That makes three modules depend on a type declared in the lowest layer, and it
puts a plain-text parser into `jplephem`, which in the Python original only
reads DAF files.

Skyfield already solved this: `skyfield/planetarylib.py` owns
`PlanetaryConstants` (text-kernel variables + binary-PCK segments) and the
`Frame` objects built from them; `skyfield/data/text_pck.py` is the text
parser; `jplephem/pck.py` is binary-only. Follow the port:

```
src/jplephem/pck.rs           binary PCK (DAF) — port of jplephem/pck.py          (#158)
src/planetarylib/mod.rs       PlanetaryConstants, RotationalElements, BodyConstants
src/planetarylib/text_pck.rs  text kernel parser — port of skyfield/data/text_pck.py (#157)
src/planetarylib/iau2015.csv  embedded WGCCRE 2015 table                          (#160)
src/planetarylib/iau_frame.rs IauFrame — impl framelib::Frame                     (#159)
src/planetarylib/pck_frame.rs PckFrame — impl framelib::Frame                     (#158)
src/planetarylib/geometry.rs  sub-points, position angles, apparent ellipse        (#163–#165)
src/planetarylib/occult.rs    occultation predicate                                (#170)
```

`planetlib::Body` gains thin accessors (`radii_km()`, `rotational_elements()`)
that delegate into the embedded table. `framelib` keeps the trait; `ItrsFrame`
goes in `framelib` next to the other Earth-centric frames.

This deviates from the issue titles (`framelib:` on #159, `jplephem:` on #157).
Update the titles when the module lands.

### 2.2 Three body-fixed frame types, one rule for Earth

| Type | Source | Accuracy | Bodies |
|---|---|---|---|
| `IauFrame` | text PCK / embedded table, WGCCRE polynomials | ~0.1° (Earth), better elsewhere | any body with elements |
| `PckFrame` | binary PCK type-2 segment | as good as the kernel | 31006/31008 (Moon PA), 3000 (ITRF93) |
| `ItrsFrame` | `Time::c_matrix()` | IERS-grade | Earth only |

**Rule:** `IauFrame::new(399, ..)` is permitted but its doc comment must say to
prefer `ItrsFrame`. `PlanetaryConstants::frame_for(body)` — the convenience
constructor focalplane will actually call — returns `ItrsFrame` for 399,
`PckFrame` for 301 if a Moon PA kernel is loaded, and `IauFrame` otherwise.
That is the only place the choice is made.

### 2.3 `RotationalElements` shape

```rust
pub struct RotationalElements {
    pub pole_ra:  [f64; 3],   // deg, deg/cy, deg/cy²
    pub pole_dec: [f64; 3],
    pub pm:       [f64; 3],   // deg, deg/day, deg/day²
    pub nut_prec_ra:  Vec<f64>,
    pub nut_prec_dec: Vec<f64>,
    pub nut_prec_pm:  Vec<f64>,
    pub nut_prec_angles: Vec<(f64, f64)>,  // (deg, deg/cy) from the barycentre id
}
impl RotationalElements {
    /// (ra, dec, w) in radians at TDB `t`.  Centuries for pole/nut-prec, days for W.
    pub fn evaluate(&self, t: &Time) -> (f64, f64, f64);
}
```

Rotation is `Rz(W) · Rx(90° − δ) · Rz(90° + α)` (ICRF → body-fixed), which is
exactly Skyfield's `rot_z(-w)·rot_x(-dec)·rot_z(-ra)` with the sign convention
of this crate's `rot_*` helpers — check that convention once, in a unit test,
before writing either frame.

### 2.4 Phase angle and the Sun

`magnitudelib` currently treats the Sun as sitting at the SSB
(`src/magnitudelib/mod.rs:64`, deliberate comment). #162 asks
`planetary_magnitude` to call the new `Position::phase_angle`, which would
fetch the true Sun. The offset is up to ~0.005 AU and **will shift magnitudes
at the millimag level**, so the "existing tests unchanged" acceptance in #162
is not achievable as written.

Decision: keep the two separate.

- `Position::phase_angle(kernel, t)` etc. use the true Sun (Skyfield does the
  same in `positionlib.phase_angle`).
- `planetary_magnitude` keeps its SSB approximation and its signature.
  Refactor it to share the private vector maths, not the public method.
  Revisit only if a consumer needs sub-millimag agreement.

### 2.5 `Position` methods vs free functions

Everything that needs only the vectors already on a `Position` becomes a method
(`angular_semi_diameter`, `north_pole_position_angle`, `sub_observer_point`).
Anything that needs the Sun takes `&mut SpiceKernel` like `apparent()` does.
`occultation` is a free function in `planetarylib::occult` because it has three
positions and no natural receiver.

## 3. Cross-cutting prerequisites (PR 0)

Do these first, in one small PR, so every later PR can be validated.

1. **Add `spiceypy` to the Python environment.** Issues #158, #159, #161 state
   acceptance against `spiceypy.pxform`. Skyfield can replace it for #158 and
   #161 (see §5) but **nothing in the venv evaluates IAU polynomial elements**,
   so #159 needs it. Pin it in a `.spiceypy-version` file, add to
   `devops/setup_pyenv.sh` and the `python-comparison` job in
   `.github/workflows/ci.yml`. The tests that use it download
   `pck00011.tpc` and are `#[ignore]`d; the checked-in golden vectors they
   produce are what CI runs.
2. **Kernel fixture policy.** `test_data/de421.bsp` (16 MB) is the only
   checked-in kernel. Do not add more. Binary PCK tests fetch
   `moon_pa_de421_1900-2050.bpc` (the file Skyfield's own loader knows) into
   `~/.cache/starfield/` and are `#[ignore]`d; golden rotation matrices at
   three epochs are checked in as Rust constants.
3. **GitHub housekeeping.** Create milestone *Resolved solar-system bodies*;
   attach #157–#170; label #165, #168, #169, #170 `good first issue`.
4. **Fix the arithmetic in #167.** A 17 000 km orbit at 0.5 AU subtends
   $17000 / (0.5 \times 1.496\times10^8) = 2.27\times10^{-4}$ rad = **46.9″**,
   not "~55″" (55″ is 0.43 AU). Correct the acceptance text.

## 4. Pull request sequence

Sizes: S ≈ half a day, M ≈ one to two days. Each PR runs `cargo fmt`,
`cargo clippy`, `cargo test`, and adds an example under `examples/` where a
new public capability appears.

### Wave 0 — independent, can start immediately (any order, in parallel)

**PR 1 — `ItrsFrame` (#161), S.**
`framelib::ItrsFrame` with `rotation_at(t) = t.c_matrix()`. Ten lines plus a
regression test asserting `ItrsFrame` and `GeographicPosition::at` agree
(they share `c_matrix`, so this is a guard, not a discovery). Validate against
Skyfield `itrs.rotation_at(t)` through the existing pybridge.

**PR 2 — Moon magnitude (#168), S.**
`magnitudelib::moon_magnitude(r, delta, ph_ang)` using the Allen / Lane–Irvine
V curve; wire `301` into the match. Document the valid phase-angle range.
Tests: full Moon −12.7 ± 0.1, first quarter −10.0 ± 0.2.

**PR 3 — PCK URLs (#169, first half), S.**
`NAIF_PCK_URL`; `resolve_url` maps `*.tpc` and `*.bpc` to it. Unit test with no
network. The typed `Loader::open_text_pck` / `open_binary_pck` helpers wait for
Wave 1 and land with PR 6.

**PR 4 — `observe_star` (#166), M.**
`Position::observe_star(star: &StarData, pm: Option<&ProperMotion>, parallax_mas: Option<f64>, epoch: &Time) -> Position`.
Apply proper motion and parallax to a large nominal distance, return
`PositionKind::Astrometric` with `observer_barycentric` set (mandatory —
`apparent()` panics without it). Port Skyfield `Star._observe_from_bcrs`.
Validate 20 Hipparcos stars against `earth.at(t).observe(star).apparent().radec()`
to < 1 mas in the existing pybridge tests.

**PR 5 — Spacecraft observer (#167), M.**
`KeplerOrbit::barycentric_at(kernel, t)`: centre body state from the kernel
plus the orbit's relative state; returns `PositionKind::Barycentric` with
velocity so aberration is right. `Position::from_spk_target(kernel, id, t)`
for negative NAIF ids — note only SPK types 2/3/21 are supported; spacecraft
kernels are commonly types 1/13, so this helper returns `UnsupportedDataType`
for those until someone needs them. Analytic test: 46.9″ offset at 0.5 AU.

**PR 6 — Illumination geometry (#162), S.**
`phase_angle`, `illuminated_fraction`, `solar_elongation` on `Position`, all
taking `&mut SpiceKernel` and using the true Sun (§2.4). Return
`Err(MissingObserver)` instead of NaN when `observer_barycentric` is `None`.
Validate against Skyfield `phase_angle` / `fraction_illuminated`. Test the
HiRISE epoch (2007-10-03): 98° ± 0.5°, 0.43 ± 0.01.

### Wave 1 — the foundation

**PR 7 — `planetarylib` skeleton, text PCK parser, embedded table (#157, #160), M.**
One PR, because the embedded table and the parser produce the same type and
test each other.

- `planetarylib::text_pck::parse(&str) -> HashMap<String, KernelValue>`,
  handling `\begindata` / `\begintext`, `=` and `+=`, parenthesised vectors,
  `D` exponents, continuation. Port of `skyfield/data/text_pck.py`.
- `PlanetaryConstants { variables, segments }` with `read_text`,
  `radii(body)`, `rotational_elements(body)`.
- `planetarylib/iau2015.csv` with a provenance header (Archinal et al. 2018 +
  2019 corrigendum); `include_str!`ed; parsed once into a `LazyLock`.
- `Body::radii_km()`, `mean_radius_km()`, `flattening()`,
  `rotational_elements()` delegating to the table.
- `Loader::open_text_pck(filename)` (the deferred half of #169).

Tests: round-trip every `BODY*` key of an embedded `pck00011.tpc` excerpt
(399, 499, 301, 599, four barycentre `NUT_PREC_ANGLES`); compare the parsed
variables with Skyfield `PlanetaryConstants.read_text(...).variables` via
pybridge; assert the embedded table equals the parsed kernel for every `Body`.

**PR 8 — Binary PCK type 2 (#158), M.**
Replace the stub in `jplephem/pck.rs` with a port of `jplephem/pck.py`:
`PckSegment { body, frame, start, end, data_type }` and
`compute(tdb_jd) -> (Vector3 angles, Vector3 rates)`. Reuse the SPK type-2
record loader. `PlanetaryConstants::read_binary`, `Loader::open_binary_pck`,
`planetarylib::PckFrame` implementing `Frame` via `Rz(W)·Rx(90°−δ)·Rz(90°+α)`.
Types other than 2 → `UnsupportedDataType`.

Validation: Skyfield `PlanetaryConstants.build_frame_named('MOON_PA_DE421').rotation_at(t)`
using `moon_pa_de421_1900-2050.bpc` + `moon_080317.tf` — both already in
Skyfield's download table, no spiceypy required. `#[ignore]`d live test plus
three checked-in golden matrices; target < 1e-9 rad.

### Wave 2 — frames and simple geometry

**PR 9 — `IauFrame` (#159), M.**
`RotationalElements::evaluate(t)` (nut-prec sums included) and
`planetarylib::IauFrame { body, elements }` implementing `Frame`.
`PlanetaryConstants::frame_for(body)` applying the §2.2 rule. The doc comment
states the longitude convention: elements give **west-positive** planetographic
longitude for Mars and most bodies; `sub_observer_point` in PR 11 exposes both.

Validation: spiceypy `pxform('J2000', 'IAU_{MARS,EARTH,MOON,JUPITER}', et)` at
three epochs, < 1″. Golden matrices checked in; the live test `#[ignore]`d.

**PR 10 — Angular size and occultation (#165, #170), S.**
`Position::angular_semi_diameter(radii_km)`,
`Position::apparent_ellipse(frame, radii_km, t) -> (a, b, pa)`,
`planetarylib::occult::occultation(observer, target, occulter, r_t, r_o) -> Occultation`.
Tests: 18.90″ Earth and 5.14″ Moon from Mars at the HiRISE epoch (< 0.02″);
Jupiter equator-on ratio 0.0649; three hand-built occultation cases; one
`find_discrete` search for a Mars-occults-Earth interval from a Mars orbit
(uses PR 5).

### Wave 3 — disk orientation

**PR 11 — Sub-observer / sub-solar points (#163), M.**
`SubPoint { lon_rad, lat_rad, planetocentric }`;
`Position::sub_observer_point(frame, radii_km, t)` and
`sub_solar_point(frame, radii_km, kernel, t)`. Rotate with the frame evaluated
at `t − light_time`, as SPICE `subpnt` with `LT+S`. Provide both
planetocentric and planetographic output; Horizons reports planetographic
with the IAU west/east convention per body.

Validation: Horizons observer table, QUANTITIES 14 and 15, Mars-from-Earth and
Earth-from-Mars at three epochs, < 0.05°. `horizons::parser` already parses
these columns; fetch once, check in the rows as fixtures.

**PR 12 — Position angles (#164), S.**
`Position::north_pole_position_angle(frame, t)` and
`bright_limb_position_angle(kernel, t)`, radians east of celestial north.
Validation: Horizons NP.ang and S-brt (QUANTITIES 17, 24), < 0.1°, same
fixtures as PR 11.

### Release

Bump to 0.15.0 after Wave 1 lands (new module, new kernel format), 0.16.0
after Wave 3. Each bump updates `CHANGELOG.md`. Update `README.md`'s module
list and `AGENTS.md`'s build/test notes for the new spiceypy dependency.

## 5. Validation matrix

| Issue | Reference | Path |
|---|---|---|
| #157 text PCK | Skyfield `PlanetaryConstants.read_text` | pybridge, CI |
| #158 binary PCK | Skyfield `build_frame_named('MOON_PA_DE421')` | pybridge + `#[ignore]` download |
| #159 IAU frame | **spiceypy** `pxform` | `#[ignore]`; golden vectors in CI |
| #160 radii | `pck00011.tpc` via #157 | CI |
| #161 ITRS | Skyfield `itrs.rotation_at` | pybridge, CI (already covered by `c_matrix` tests) |
| #162 phase | Skyfield `phase_angle`, `fraction_illuminated` | pybridge, CI |
| #163–#165 | JPL Horizons observer tables | checked-in fixtures, CI |
| #166 stars | Skyfield `observe(star).apparent()` | pybridge, CI |
| #167, #170 | analytic / hand-built | CI |
| #168 Moon mag | literature values | CI |

## 6. Dependency graph

```
PR0 prerequisites
 ├─ PR1 ItrsFrame ─────────────────────────────────────┐
 ├─ PR2 Moon mag                                       │
 ├─ PR3 PCK URLs                                       │
 ├─ PR4 observe_star                                   │
 ├─ PR5 spacecraft observer ──────────────┐            │
 ├─ PR6 phase angle ──────────────┐       │            │
 └─ PR7 planetarylib + text PCK + table   │            │
      ├─ PR8 binary PCK / PckFrame        │            │
      └─ PR9 IauFrame  ◄──────────────────┼────────────┘  (frame_for needs all three)
           ├─ PR10 angular size, occultation ◄─ PR5
           ├─ PR11 sub-points ◄─ PR6
           └─ PR12 position angles ◄─ PR6
```

Critical path: **PR0 → PR7 → PR9 → PR11**. Wave 0 PRs can be reviewed while
PR7 is in flight.

## 7. Corrections to file back on the issues

- **#157, #159** — retitle to `planetarylib:`; `RotationalElements` lives there (§2.1).
- **#160** — no existing `include_str!` CSV pattern for catalogs; this PR establishes one.
- **#161** — `Time::c_matrix()` already is the requested rotation; the issue is a wrapper.
- **#162** — `planetary_magnitude` will *not* call the public method (§2.4); strike that line.
- **#166** — `ProperMotion` is `framelib::inertial::ProperMotion` (re-exported at the crate root); `StarData` is `catalogs::StarData`. Use those, not `starlib::Star`.
- **#167** — 46.9″, not 55″ (§3.4). Note SPK type limitation for spacecraft kernels.
- **#169** — `resolve_url` half is Wave 0; typed helpers land with PR 7/8.
- **#170** — module is `planetarylib::occult`.
- **#158, #159, #161** — spiceypy is required only for #159; the other two validate with Skyfield.

## 8. Open questions for the maintainer

1. Accept `spiceypy` as a pinned test-only Python dependency? (Alternative:
   validate #159 indirectly through Horizons sub-points in PR 11 and skip a
   direct frame test. Not recommended — a direct test catches sign errors that
   sub-point comparison can mask.)
2. Is `planetarylib` the right name, or should this go under `planetlib`?
   The Skyfield port argument favours `planetarylib`; the existing
   `planetlib` already has `Body`, so the two will be adjacent either way.
3. Does focalplane need `rotation_and_rate_at` (angular velocity of the
   frame) for motion-blur of limb features? Skyfield has it; it is cheap to add
   in PR 8/9 and hard to bolt on later.
