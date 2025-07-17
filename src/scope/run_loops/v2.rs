use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use log::{error, trace, debug};
use rusb::DeviceHandle;
use crate::PowerStatus;
use crate::scope::commands::{Command, ScopeCommand};
use crate::scope::StatusResponse;

impl crate::Nlab {
    pub(crate) fn run_v2(
        usb_device: DeviceHandle<rusb::GlobalContext>,
        command_tx: Sender<Command>,
        command_rx: Receiver<Command>,
        fw_version: Arc<RwLock<Option<u16>>>,
        power_status: Arc<RwLock<PowerStatus>>,
    ) {
        let mut active_comms_request: Option<(u8, Command)> = None;
        let mut active_data_request: Option<(u8, Command)> = None;
        let mut incoming_usb_buffer: [u8; 64] = [0u8; 64];
        let mut outgoing_usb_buffer: [u8; 64] = [0u8; 64];
        let mut incoming_channel_buffers: [[u8; 64]; 4] = [[0u8; 64]; 4];
        let mut request_id: u8 = 0;

        'communication: loop {
            // Check first to see if we have a cancelled active request
            if let Some((id, Command::RequestData(rq))) = &active_data_request {
                // we get the active request
                if let Ok(()) = rq.stop_recv.try_recv() {
                    // We have received a stop signal
                    command_tx.send(Command::StopData).unwrap();
                    debug!("Sent a stop command to request {id}");
                }
            }

            if active_comms_request.is_none() {
                if let Ok(command) = command_rx.try_recv() {
                    // If we have a command from the front-end, assign a new requestID
                    // and send the request out to the usb control line

                    outgoing_usb_buffer.fill(0);
                    request_id = request_id.wrapping_add(1);
                    if request_id == 0 {
                        request_id += 1
                    }

                    outgoing_usb_buffer[0] = request_id;
                    outgoing_usb_buffer[1] = command.id_byte();
                    debug!("Sent request {}: command: {}", request_id, command.id_byte());

                    // Fill the outgoing buffer with whatever we need
                    match &command {
                        Command::Quit => { break 'communication; }
                        Command::Initialize(power_on, _) => {
                            outgoing_usb_buffer[2] = *power_on as u8;
                            active_comms_request = Some((request_id, command));
                        }
                        Command::SetAnalogOutput(cmd) => {
                            cmd.fill_tx_buffer(&mut outgoing_usb_buffer)
                                .expect("Invalid parameters given to AxRequest");
                            active_comms_request = Some((request_id, command));
                        }
                        Command::SetPulseOutput(cmd) => {
                            cmd.fill_tx_buffer(&mut outgoing_usb_buffer)
                                .expect("Invalid parameters given to PxRequest");
                            active_comms_request = Some((request_id, command));
                        }
                        Command::RequestData(cmd) => {
                            cmd.fill_tx_buffer(&mut outgoing_usb_buffer)
                                .expect("Invalid parameters given to DataRequest");
                            active_comms_request = Some((request_id, command));
                        }
                        Command::StopData => {
                            active_comms_request = Some((request_id, command));
                        }
                    };

                    if let Err(error) = usb_device.write_bulk(0x01,
                                                              &outgoing_usb_buffer,
                                                              Duration::from_millis(100))
                    {
                        error!("USB write error: {error:?}");
                        break 'communication;
                    }
                }
            }

            match usb_device.read_bulk(0x81,
                                       &mut incoming_usb_buffer,
                                       Duration::from_millis(1))
            {
                Err(rusb::Error::Timeout) => {}
                Ok(_) => {
                    let response = StatusResponse::new(&incoming_usb_buffer);

                    *fw_version.write().unwrap() = Some(response.fw_version);
                    power_status.write().unwrap().state = response.power_state;
                    power_status.write().unwrap().usage = response.power_usage as f64 / 1000.0 * 5.0;

                    if response.request_id == 0 {
                        trace!("Received a status update from nLab");
                    } else if let Some((id, command)) = &active_comms_request {
                        // If we have an active request with this ID
                        if *id == response.request_id {

                            // Handle the incoming usb packet
                            command.handle_rx(&incoming_usb_buffer);

                            // If the command has finished it's work
                            if command.is_finished() {
                                debug!("Finished request ID: {request_id}");
                                if let Command::StopData = command {
                                    active_data_request = None;
                                }
                            } else {
                                debug!("Received request ID: {request_id}");
                            }

                            if let Command::RequestData(_) = command {
                                debug!("Setting Active Data Request: {request_id}");
                                active_data_request = active_comms_request.take();
                            } else {
                                active_comms_request = None;
                            }
                        }
                    } else {
                        error!("Received response for request {}, but cannot find a record of that request", response.request_id);
                    }
                }
                Err(error) => {
                    error!("USB read error: {error:?}");
                    break 'communication;
                }
            }

            let mut received_ch_data = false;

            if let Some((request_id, Command::RequestData(data_request))) = &active_data_request {
                for (ch, &ep) in [0x82u8, 0x83u8, 0x84u8, 0x85u8].iter().enumerate() {
                    let buf = &mut incoming_channel_buffers[ch];
                    if data_request.channels[ch].is_on {
                        match usb_device.read_bulk(ep, buf, Duration::from_millis(1))
                        {
                            Err(rusb::Error::Timeout) => {}
                            Ok(_) => {
                                let received_request_id = buf[0];
                                debug!("Received data for request {received_request_id}, active request {request_id}");
                                if received_request_id == *request_id {
                                    data_request.handle_incoming_data(buf, ch);
                                    received_ch_data = true;
                                }
                            }
                            Err(error) => {
                                error!("USB read error: {error:?}");
                                break 'communication;
                            }
                        }
                    }
                }
            }


            // If we received data on any incoming channel, collate any results
            if received_ch_data {
                if let Some((request_id, Command::RequestData(data_request))) = &active_data_request {
                    data_request.collate_results();
                    if data_request.is_finished() {
                        debug!("Finished request ID: {request_id}");
                        active_data_request = None;
                    }
                }
            }
        }
    }
}