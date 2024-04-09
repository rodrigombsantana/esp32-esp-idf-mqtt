
//libs gerais
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals}, sys};
//libs para wifi
use esp_idf_svc::{nvs::EspDefaultNvsPartition, wifi::{ClientConfiguration, Configuration, EspWifi}};
//libs para mqtt
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration};
use embedded_svc::mqtt::client::{EventPayload,QoS};


const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

fn main() {
    sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Cliente MQTT com esp-idf");
    

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    
    let mut wifi_driver = EspWifi::new(peripherals.modem, sysloop, Some(nvs)).unwrap();
    
// Configuração do cliente WiFi
    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    })).unwrap();
     // Início do WiFi
    wifi_driver.start().unwrap();
    println!("Wifi Iniciado? : {:?}", wifi_driver.is_started());    
    //configura o LED
    let mut led = PinDriver::output(peripherals.pins.gpio2).unwrap();

    
    println!("Wifi Conectando... {:?}", wifi_driver.connect());

    // wait to get connected
    //println!("Wait to get connected");
    let mut c =1;
    loop {
        c+=1;
        println!("Tentativa de Conexao #{c}");
        let res = wifi_driver.is_connected();
        match res {
            Ok(connected) => {
                if connected {
                    break;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                loop {}
            }
        }
        FreeRtos::delay_ms(1000u32);
    }
    println!("{:?}", wifi_driver.is_connected());

    
    
    
    // wait for getting an ip address
    println!("Wait to get an ip address");
    c=0;
    loop {
        c+=1;
        println!("Tentativa de obter IP do DHCP #{c}");
        let res = wifi_driver.is_up();
        match res {
            Ok(connected) => {
                if connected {
                    let ip =wifi_driver.sta_netif().get_ip_info();
                    println!("IP criado. {:?}", ip);
                    led.set_high().unwrap(); //liga LED para indicar wifi conectada
                    break;
                    
                }
            }
            Err(err) => {
                println!("{:?}", err);
                loop {}
                }
        }
        FreeRtos::delay_ms(1000u32);
    }
    println!("inicia configuracao mqtt");
    //inicia configuracao mqtt
    let mqtt_config = MqttClientConfiguration::default();
    let mqtt_url = "mqtt://mqtt.eclipseprojects.io";    
    let client = EspMqttClient::new_cb(
        mqtt_url,
        &mqtt_config,
        move |message_event| {
            match message_event.payload(){
                EventPayload::Connected(_) => {
                    println!("Connectado a {mqtt_url}");
                },
                EventPayload::Subscribed(id) => println!("Inscrito com id {id}"),
                EventPayload::Received{data, ..} => {
                    if !data.is_empty() {
                        led.toggle().unwrap();
                        println!("Mensagem recebida:  {}", std::str::from_utf8(data).unwrap());
                        FreeRtos::delay_ms(500u32);
                        led.toggle().unwrap();
                    }
                }
                _ => println!("Erro conectando a {mqtt_url}!"),
            };
        },
    );
    let mut client = client.unwrap();
    client.subscribe("profrs/led",QoS::AtLeastOnce).expect("erro ao subscrever no tópico!");

    println!("Esperando mensagem...");
    loop {
        FreeRtos::delay_ms(1000u32);
    }

}

