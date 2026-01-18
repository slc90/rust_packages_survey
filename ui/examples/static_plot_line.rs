use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, setup)
		.run();
}

// 正弦波参数
const POINT_COUNT: usize = 4096; // 点数
const WAVE_CYCLES: f32 = 100.0; // 显示的周期数
const AMPLITUDE: f32 = 100.0; // 振幅
const WIDTH: f32 = 1920.0; // 显示宽度

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	// 创建 2D 相机
	commands.spawn(Camera2d);

	// 生成正弦波顶点数据
	let mut points = Vec::with_capacity(POINT_COUNT);

	for i in 0..POINT_COUNT {
		// X 坐标：从 -WIDTH/2 到 WIDTH/2
		let x = (i as f32 / (POINT_COUNT - 1) as f32 - 0.5) * WIDTH;

		// 计算正弦波值
		// 将 x 映射到 [-π * WAVE_CYCLES, π * WAVE_CYCLES]
		let phase = (x / WIDTH) * 2.0 * std::f32::consts::PI * WAVE_CYCLES;
		let y = phase.sin() * AMPLITUDE;

		points.push(Vec2::new(x, y));
	}

	// 创建 Polyline2d 并将其转换为 Mesh
	let polyline = Polyline2d::new(points);
	let mesh_handle = meshes.add(polyline);

	// 创建颜色材质（蓝色）
	let color = Color::Srgba(Srgba::new(0.2, 0.6, 0.9, 1.0));
	let material_handle = materials.add(color);

	// 创建折线实体
	commands.spawn((
		Mesh2d(mesh_handle),
		MeshMaterial2d(material_handle),
		Transform::from_xyz(0.0, 0.0, 0.0),
	));

	// 可选：添加坐标轴和网格线（辅助可视化）
	add_visualization_aids(&mut commands, &mut meshes, &mut materials);
}

/// 添加坐标轴和网格线辅助可视化
fn add_visualization_aids(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<ColorMaterial>>,
) {
	// 创建 X 轴
	let x_axis_points = vec![Vec2::new(-400.0, 0.0), Vec2::new(400.0, 0.0)];
	let x_axis_polyline = Polyline2d::new(x_axis_points);
	let x_axis_mesh = meshes.add(x_axis_polyline);
	let x_axis_color = materials.add(Color::Srgba(Srgba::new(0.5, 0.5, 0.5, 1.0)));

	commands.spawn((
		Mesh2d(x_axis_mesh),
		MeshMaterial2d(x_axis_color),
		Transform::from_xyz(0.0, 0.0, -0.1), // 稍微靠后，避免遮挡波形
	));

	// 创建 Y 轴
	let y_axis_points = vec![Vec2::new(0.0, -100.0), Vec2::new(0.0, 100.0)];
	let y_axis_polyline = Polyline2d::new(y_axis_points);
	let y_axis_mesh = meshes.add(y_axis_polyline);
	let y_axis_color = materials.add(Color::Srgba(Srgba::new(0.5, 0.5, 0.5, 1.0)));

	commands.spawn((
		Mesh2d(y_axis_mesh),
		MeshMaterial2d(y_axis_color),
		Transform::from_xyz(0.0, 0.0, -0.1),
	));

	// 添加文本标签（使用简单的线标记）
	add_axis_ticks(commands, meshes, materials);
}

/// 添加坐标轴刻度标记
fn add_axis_ticks(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<ColorMaterial>>,
) {
	// X 轴刻度
	for i in -4..=4 {
		if i == 0 {
			continue; // 跳过原点
		}
		let x = i as f32 * 100.0;
		let tick_points = vec![Vec2::new(x, -5.0), Vec2::new(x, 5.0)];
		let tick_polyline = Polyline2d::new(tick_points);
		let tick_mesh = meshes.add(tick_polyline);
		let tick_color = materials.add(Color::Srgba(Srgba::new(0.7, 0.7, 0.7, 1.0)));

		commands.spawn((
			Mesh2d(tick_mesh),
			MeshMaterial2d(tick_color),
			Transform::from_xyz(0.0, 0.0, -0.2),
		));
	}

	// Y 轴刻度
	for i in -1..=1 {
		if i == 0 {
			continue; // 跳过原点
		}
		let y = i as f32 * 100.0;
		let tick_points = vec![Vec2::new(-5.0, y), Vec2::new(5.0, y)];
		let tick_polyline = Polyline2d::new(tick_points);
		let tick_mesh = meshes.add(tick_polyline);
		let tick_color = materials.add(Color::Srgba(Srgba::new(0.7, 0.7, 0.7, 1.0)));

		commands.spawn((
			Mesh2d(tick_mesh),
			MeshMaterial2d(tick_color),
			Transform::from_xyz(0.0, 0.0, -0.2),
		));
	}
}
