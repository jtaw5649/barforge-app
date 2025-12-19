use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use waybar_manager::app::state::{SortField, SortOrder};
use waybar_manager::testing::{RegistryModuleBuilder, test_app_with_registry};
use waybar_registry_types::ModuleCategory;

fn generate_modules(count: usize) -> Vec<waybar_registry_types::RegistryModule> {
    let categories = [
        ModuleCategory::System,
        ModuleCategory::Network,
        ModuleCategory::Media,
        ModuleCategory::Weather,
        ModuleCategory::Hardware,
        ModuleCategory::Custom,
    ];

    (0..count)
        .map(|i| {
            RegistryModuleBuilder::new(&format!("module-{i}"))
                .author(&format!("author-{}", i % 50))
                .category(categories[i % categories.len()])
                .downloads((count - i) as u64 * 100)
                .tags(vec!["tag1", "tag2", &format!("tag-{}", i % 10)])
                .build()
        })
        .collect()
}

fn bench_filtered_modules(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_modules");

    for size in [100, 500, 1000] {
        let modules = generate_modules(size);
        let app = test_app_with_registry(modules);

        group.bench_with_input(BenchmarkId::new("no_filter", size), &app, |b, app| {
            b.iter(|| black_box(app.filtered_modules()))
        });
    }

    group.finish();
}

fn bench_filtered_modules_with_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_modules_search");

    for size in [100, 500, 1000] {
        let modules = generate_modules(size);
        let mut app = test_app_with_registry(modules);
        app.browse.search_query = "module-5".to_string();

        group.bench_with_input(BenchmarkId::new("search_query", size), &app, |b, app| {
            b.iter(|| black_box(app.filtered_modules()))
        });
    }

    group.finish();
}

fn bench_filtered_modules_with_category(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_modules_category");

    for size in [100, 500, 1000] {
        let modules = generate_modules(size);
        let mut app = test_app_with_registry(modules);
        app.browse.selected_category =
            waybar_manager::app::state::CategoryFilter(Some(ModuleCategory::System));

        group.bench_with_input(BenchmarkId::new("category_filter", size), &app, |b, app| {
            b.iter(|| black_box(app.filtered_modules()))
        });
    }

    group.finish();
}

fn bench_filtered_modules_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_modules_sorting");

    let modules = generate_modules(500);

    for (name, sort_field) in [
        ("by_name", SortField::Name),
        ("by_downloads", SortField::Downloads),
        ("by_rating", SortField::Rating),
    ] {
        let mut app = test_app_with_registry(modules.clone());
        app.browse.sort_field = sort_field;
        app.browse.sort_order = SortOrder::Descending;

        group.bench_with_input(BenchmarkId::new(name, 500), &app, |b, app| {
            b.iter(|| black_box(app.filtered_modules()))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_filtered_modules,
    bench_filtered_modules_with_search,
    bench_filtered_modules_with_category,
    bench_filtered_modules_sorting
);
criterion_main!(benches);
