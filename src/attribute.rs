//! Attribute definitions

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{pair, separated_pair},
    Parser,
};

use crate::{color::Color, ir::ParseResult, ws::ws};

pub type Double = f64;
pub type Int = i64;

#[derive(Debug)]

pub enum Attribute {
    Background(String),
    ArrowHead(ArrowType),
    ArrowSize(Double),
    ArrowTail(ArrowType),
    Bb(Rectangle),
    BgColor(ColorAttribute),
    Center(bool),
    Charset(String),
    Color(ColorAttribute),
    ColorsSheme(String),
    Comment(String),
    Concentrate(bool),
    Decorate(bool),
    Dir(DirType),
    Distortion(Double),
    FillColor(ColorAttribute),
    FixedSize(FixedSize),
    FontColor(Color),
    FontName(String),
    FontPath(String),
    FontSize(Double),
    ForceLabels(bool),
    GradientAngle(Int),
    HeadClip(bool),
    HeadLabel(LabelString),
    Height(Double),
    Image(String),
    ImagePath(String),
    ImagePos(String),
    ImageScale(ImageScale),
    Label(LabelString),
    LabelAngle(Double),
    LabelDistance(Double),
    LabelFloat(bool),
    LabelFontColor(Color),
    LabelFontName(String),
    LabelFontSize(Double),
    LabelJust(TextJustification),
    LabelLoc(TextLocation),
    Landscape(bool),
    Layer(Vec<String>),
    LayerListSep(String),
    Layers(Vec<String>),
    LayerSelect(Vec<String>),
    LayerSep(String),
    Layout(String),
    Margin(Point),
    NodeSep(Double),
    NoJustify(bool),
    Orientation(Orientation),
    OutputOrder(OutputMode),
    Pack(Pack),
    PackMode(PackMode),
    Pad(Point),
    Page(Point),
    PageDir(PageDir),
    PenColor(Color),
    PenWidth(Double),
    Peripheries(Int),
    Pos(Position),
    Quantum(Double),
    Ratio(Ratio),
    Rects(Rectangle),
    Regular(bool),
    Rotate(Int),
    SamplePoints(Int),
    Shape(Shape),
    ShapeFile(String),
    Sides(Int),
    Size(Point),
    Skew(Double),
    SortV(Int),
    Splines(EdgeRespresentation),
    Style(Vec<Style>),
    TailLp(Point),
    TailClip(bool),
    TailLabel(LabelString),
    Vertices(Vec<Point>),
    ViewPort(ViewPort),
    Weight(Double),
    Width(Double),
    XLabel(LabelString),
    Z(Double),
}

#[derive(Debug)]
pub enum ArrowType {
    Normal,
    Inv,
    Dot,
    InvDot,
    ODot,
    InvODot,
    None,
    Tee,
    Empty,
    InvEmpty,
    Diamond,
    ODiamond,
    EDiamond,
    Crow,
    Box,
    OBox,
    Open,
    HalfOpen,
    Vee,
}

#[derive(Debug)]
pub struct Rectangle {
    lower_left: Point,
    upper_right: Point,
}

#[derive(Debug)]
pub struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
pub enum ColorAttribute {
    Color(Color),
    ColorList(Vec<Color>),
}

#[derive(Debug)]
pub enum ClusterMode {
    Local,
    Global,
    None,
}

#[derive(Debug)]
pub enum DirType {
    Forward,
    Back,
    Both,
    None,
}

#[derive(Debug)]
pub enum ImageScale {
    False,
    True,
    Width,
    Height,
    Both,
}

#[derive(Debug)]
pub enum TextJustification {
    Left,
    Right,
    Center,
}

#[derive(Debug)]
pub enum TextLocation {
    Top,
    Bottom,
    Center,
}

#[derive(Debug)]
pub enum Orientation {
    Landscape,
    Rotation(Double),
}

#[derive(Debug)]
pub enum Pack {
    True,
    False,
    Value(Int),
}

type LabelString = String;

#[derive(Debug)]
pub enum FixedSize {
    True,
    False,
    Shape,
}

#[derive(Debug)]
pub enum OutputMode {
    BreadthFirst,
    NodesFirst,
    EdgesFirst,
}

#[derive(Debug)]
pub enum PackMode {
    Node,
    Clust,
    Graph,
    Array { size: Int, flags: Vec<PackFlag> },
}

#[derive(Debug)]
pub enum PackFlag {
    ColumnMajor,
    Top,
    Bottom,
    Left,
    Right,
    User,
}

#[derive(Debug)]
pub struct PageDir {
    primary: TraversalDir,
    secondary: TraversalDir,
}

#[derive(Debug)]
pub enum TraversalDir {
    Vertical(VerticalDir),
    Horizontal(HorizontalDir),
}

#[derive(Debug)]
pub enum VerticalDir {
    BottomToTop,
    TopToBottom,
}

#[derive(Debug)]
pub enum HorizontalDir {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug)]
pub enum Position {
    Point(Point),
    Spline(Vec<Point>),
}

#[derive(Debug)]
pub enum Ratio {
    Numeric(Double),
    Fill,
    Compress,
    Expand,
    Auto,
}

// only polygon shapes currently supported
#[derive(Debug)]
pub enum Shape {
    Box,
    Polygon,
    Ellipse,
    Oval,
    Circle,
    Point,
    Egg,
    Triangle,
    Plaintext,
    Plain,
    Diamond,
    Trapezium,
    Parallelogram,
    House,
    Pentagon,
    Hexagon,
    Septagon,
    Octagon,
    DoubleCircle,
    DoubleOctagon,
    TripleOctagon,
    InvTriangle,
    InvTrapezium,
    InvHouse,
    MDiamond,
    MSquare,
    MCircle,
    Rect,
    Rectangle,
    Square,
    Star,
    None,
    Underline,
    Cylinder,
    Note,
    Tab,
    Folder,
    Box3d,
    Component,
    Promoter,
    Cds,
    Terminator,
    Utr,
    PrimerSite,
    RestrictionSite,
    FivePOverhang,
    ThreePOverhang,
    NoOverhang,
    Assembly,
    Signature,
    Insulator,
    Ribosite,
    RnaStab,
    ProteaseSite,
    ProteinStab,
    RPromoter,
    RArrow,
    LArrow,
    LPromoter,
}

fn shape_parser(input: &str) -> ParseResult<&str, Shape> {
    ws(alt((
        tag("box").map(|_| Shape::Box),
        tag("polygon").map(|_| Shape::Polygon),
        tag("ellipse").map(|_| Shape::Ellipse),
        tag("oval").map(|_| Shape::Oval),
        tag("circle").map(|_| Shape::Circle),
        tag("point").map(|_| Shape::Point),
    )))(input)
}

#[derive(Debug)]
pub enum EdgeRespresentation {
    Spline,
    LineSegment,
    Off,
    Polyline,
    Ortho,
    Curved,
}

#[derive(Debug)]
pub enum Style {
    Dashed,
    Dotted,
    Solid,
    Invis,
    Bold,
    Tapered,
    Filled,
    Striped,
    Wedged,
    Diagonals,
    Rounded,
}

fn style_parser(input: &str) -> ParseResult<&str, Style> {
    ws(alt((
        tag("dashed").map(|_| Style::Dashed),
        tag("dotted").map(|_| Style::Dotted),
        tag("solid").map(|_| Style::Solid),
        tag("invis").map(|_| Style::Invis),
        tag("bold").map(|_| Style::Bold),
        tag("tapered").map(|_| Style::Tapered),
        tag("filled").map(|_| Style::Filled),
        tag("striped").map(|_| Style::Striped),
        tag("wedged").map(|_| Style::Wedged),
        tag("diagonals").map(|_| Style::Diagonals),
        tag("rounded").map(|_| Style::Rounded),
    )))(input)
}

fn styles_parser(input: &str) -> ParseResult<&str, Vec<Style>> {
    separated_list1(char(','), style_parser)(input)
}

#[derive(Debug)]
pub struct ViewPort {
    width: Double,
    height: Double,
    zoom: Double,
    center: ViewPortCenter,
}

#[derive(Debug)]
pub enum ViewPortCenter {
    Position(Point),
    NodeName(String),
}

pub enum AttributeParseError {
    AttributeNameNotFound,
    InvalidAttribueValue,
}

pub fn attribute_parser(input: &str) -> ParseResult<&str, Attribute> {
    let (rest, (attr, _)) = pair(
        alt((
            map(
                separated_pair(ws(tag("style")), char('='), ws(styles_parser)),
                |(_, style)| Attribute::Style(style),
            ),
            map(
                separated_pair(ws(tag("shape")), char('='), ws(shape_parser)),
                |(_, shape)| Attribute::Shape(shape),
            ),
        )),
        opt(ws(alt((char(','), char(';'))))),
    )(input)?;
    Ok((rest, attr))
}
