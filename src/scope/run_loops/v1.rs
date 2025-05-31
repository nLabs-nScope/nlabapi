use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use hidapi::HidDevice;
use log::{error, trace};
use crate::PowerStatus;
use crate::scope::{commands, StatusResponseLegacy};
use crate::scope::commands::Command;


impl crate::Nlab {
    pub(crate) fn run_v1(
        hid_device: HidDevice,
        command_tx: Sender<Command>,
        command_rx: Receiver<Command>,
        fw_version: Arc<RwLock<Option<u16>>>,
        power_status: Arc<RwLock<PowerStatus>>,
    ) {
        let mut active_requests_map: HashMap<u8, Command> = HashMap::new();
        let mut active_data_request: Option<u8> = None;
        let mut incoming_usb_buffer: [u8; 64] = [0u8; 64];
        let mut outgoing_usb_buffer: [u8; 65] = [0u8; 65];
        let mut request_id: u8 = 0;

        'communication: loop {
            // Check first to see if we have a cancelled active request
            if let Some(id) = &active_data_request {
                // We have an active request id
                if let Command::RequestData(rq) = active_requests_map.get(id).unwrap() {
                    // we get the active request
                    if let Ok(()) = rq.stop_recv.try_recv() {
                        // We have received a stop signal
                        command_tx.send(Command::StopData).unwrap();
                    }
                }
            }


            // check for an incoming command from the user
            // Do one of the following:
            // 1. Write a request to do the command
            // 2. Write a null packet to request an update on the power status

            if let Ok(mut command) = command_rx.try_recv() {
                if let Command::Quit = &command {
                    break 'communication;
                }

                // Process the command
                // 1. fill the outgoing USB buffer
                outgoing_usb_buffer.fill(0);
                let result = command.fill_tx_buffer_legacy(&mut outgoing_usb_buffer);
                if result.is_ok() {
                    // If we can successfully create a request packet, then
                    // 2. increment the request id
                    // 3. send the request packet
                    {
                        //TODO: make this block more concise
                        request_id = request_id.wrapping_add(1);
                        if request_id == 0 {
                            request_id += 1
                        }
                        outgoing_usb_buffer[2] = request_id;
                    }
                    if hid_device.write(&outgoing_usb_buffer).is_err() {
                        eprintln!("USB write error, ending nLab connection");
                        break 'communication;
                    }

                    if let Command::RequestData(_) = &command {
                        active_data_request = Some(request_id);
                    }
                    active_requests_map.insert(request_id, command);
                    trace!("Sent request {}", request_id);
                } else {
                    // If we cannot successfully create a request packet, then
                    // 1. Print the error
                    // 2. send a null request for status
                    eprintln!("{:?}", result);
                    if hid_device.write(&commands::NULL_REQ).is_err() {
                        eprintln!("USB write error, ending nLab connection");
                        break 'communication;
                    }
                }
            } else if hid_device.write(&commands::NULL_REQ).is_err() {
                eprintln!("USB write error, ending nLab connection");
                break 'communication;
            }

            // Read the incoming command and process it
            if hid_device.read(&mut incoming_usb_buffer).is_err() {
                eprintln!("USB read error, ending nLab connection");
                break 'communication;
            }

            let response = StatusResponseLegacy::new(&incoming_usb_buffer);

            let version = response.fw_version as u16;
            *fw_version.write().unwrap() = Some(version);
            power_status.write().unwrap().state = response.power_state;
            power_status.write().unwrap().usage = response.power_usage as f64 * 5.0 / 255.0;

            // close out request if it's open
            if response.request_id > 0 {

                // If we have an active request with this ID
                if let Some(command) = active_requests_map.get(&response.request_id)
                {
                    // Handle the incoming usb packet
                    command.handle_rx_legacy(&incoming_usb_buffer);

                    // If the command has finished its work
                    if command.is_finished() {

                        // Set the active data request as none if we just finished it
                        active_data_request = active_data_request.filter(|&id| id != response.request_id);

                        // Remove this request from the active map
                        if let Some(Command::StopData) = active_requests_map.remove(&response.request_id) {
                            // If we received the ACK on a stop command, check if we have an active id
                            if let Some(active_id) = &active_data_request {
                                // Look up that ID, remove the command from the active map
                                if let Some(Command::RequestData(rq)) = active_requests_map.remove(active_id) {
                                    // If that command is a request data command
                                    *rq.remaining_samples.write().unwrap() = 0;
                                }
                                active_data_request = None;
                            }
                        }

                        trace!("Finished request ID: {}, ADRQ: {:?}", response.request_id, active_data_request);
                    } else {
                        trace!("Received request ID: {}, ADRQ: {:?}", response.request_id, active_data_request);
                    }
                } else {
                    error!("Received response for request {}, but cannot find a record of that request", response.request_id);
                }
            }
        }
    }
}