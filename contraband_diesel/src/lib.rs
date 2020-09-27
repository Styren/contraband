use diesel::r2d2::ConnectionManager;
use contraband::config::get_prop;
use contraband::module::{ModuleFactory, Module};
use contraband::graph::Value;
use diesel::connection::Connection;
use diesel_migrations::MigrationConnection;

#[derive(Debug)]
struct TestTransaction;

impl<T: Connection + 'static> diesel::r2d2::CustomizeConnection<T, diesel::r2d2::Error> for TestTransaction {
    fn on_acquire(&self, conn: &mut T) -> Result<(), diesel::r2d2::Error> {
        conn.begin_test_transaction().unwrap();
        Ok(())
    }
}

pub struct DieselPoolModule<T>(std::marker::PhantomData<T>);

impl<T: MigrationConnection + 'static> ModuleFactory for DieselPoolModule<T> {
    fn get_module() -> Module {
        let connspec: String = get_prop("diesel", "connection_url").expect("missing database url");
        let manager = ConnectionManager::<T>::new(connspec);
        let mut pool_builder: r2d2::Builder<ConnectionManager<T>> = r2d2::Pool::builder()
            .max_size(get_prop("diesel", "max_pool_size").unwrap_or(10));
        if cfg!(test) {
            pool_builder = pool_builder.connection_customizer(Box::new(TestTransaction));
        }
        let pool = Value(
            pool_builder
            .build(manager)
            .unwrap()
        );

        let migration_conn: &T = &*pool.get().unwrap();
        if let Err(err) = diesel_migrations::run_pending_migrations(migration_conn) {
            panic!(err);
        }

        Module::new()
            .export_val(&pool)
            .provide_value(pool)
    }
}
