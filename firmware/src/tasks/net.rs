use crate::hardware::Ethernet;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, Ethernet>) -> ! {
    runner.run().await
}
