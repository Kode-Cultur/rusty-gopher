/*
 *    Copyright (C) 2016 Stefan Luecke
 *
 *    This program is free software: you can redistribute it and/or modify
 *    it under the terms of the GNU Affero General Public License as published
 *    by the Free Software Foundation, either version 3 of the License, or
 *    (at your option) any later version.
 *
 *    This program is distributed in the hope that it will be useful,
 *    but WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *    GNU Affero General Public License for more details.
 *
 *    You should have received a copy of the GNU Affero General Public License
 *    along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 *    Authors: Stefan Luecke <glaxx@glaxx.net>
 */

extern crate libc;
extern crate std;

pub fn switch_to_uid(uid: libc::uid_t) -> Result<libc::uid_t, &'static str> {
    let olduid = get_uid();
    if olduid == uid {
        return Ok(olduid);
    }

    let setuidres;
    unsafe {
        setuidres = libc::setuid(uid);
    }
    if setuidres != 0 {
        Err("Error setting uid")
    } else {
        Ok(get_uid())
    }
}

pub fn get_uid_by_name(user: std::string::String) -> Result<libc::uid_t, &'static str> {
    let uid;
    unsafe {
        let desired_user_name = std::ffi::CString::new(user.clone()).unwrap();
        let desired_user_name_ptr = desired_user_name.as_ptr();
        let desired_user_passwd = libc::getpwnam(desired_user_name_ptr);

        if desired_user_passwd.is_null() {
            return Err("User not found");
        }
        uid = (*desired_user_passwd).pw_uid;
    }
    Ok(uid)
}

pub fn get_uid() -> libc::uid_t {
    let uid;
    unsafe {
        uid = libc::getuid();
    }
    uid
}

pub fn get_canonical_hostname() -> std::string::String {
    let hostnamelen: libc::c_long;
    unsafe {
        hostnamelen = libc::sysconf(libc::_SC_HOST_NAME_MAX) + 1; // +1 for the trailing \0
    }
    let mut hostnamevec = vec![0 as u8; hostnamelen as usize];

    unsafe {
        libc::gethostname(hostnamevec.as_mut_ptr() as *mut i8, hostnamelen as usize);
    }

    let mut resultvec: Vec<u8> = std::vec::Vec::new();
    for c in hostnamevec {
        if c == 0 {
            break;
        } else {
            resultvec.push(c);
        }
    }
    let mut hostname = std::ffi::CString::new(resultvec).unwrap();

    unsafe {
        let hints = libc::addrinfo {
            ai_family: libc::AF_UNSPEC,
            ai_socktype: libc::SOCK_STREAM,
            ai_flags: libc::AI_CANONNAME,
            ai_addr: 0 as *mut libc::sockaddr,
            ai_protocol: 0,
            ai_addrlen: 0,
            ai_canonname: 0 as *mut i8,
            ai_next: 0 as *mut libc::addrinfo,
        };
        let mut gai_info: *mut libc::addrinfo = 0 as *mut libc::addrinfo;
        let gai_service = std::ffi::CString::new("gopher").unwrap();
        let res = libc::getaddrinfo(
            hostname.as_ptr(),
            gai_service.as_ptr(),
            &hints,
            &mut gai_info,
        );
        if res != 0 {
            panic!("{}", res);
        }
        if gai_info != 0 as *mut libc::addrinfo {
            // from_raw takes ownership of the pointer
            let temp = std::ffi::CString::from_raw((*gai_info).ai_canonname);

            hostname = temp.clone();

            // into_raw releases the ownership of the pointer as it is managed by C code
            temp.into_raw();
        }
        libc::freeaddrinfo(gai_info);
    }
    hostname.into_string().unwrap()
}
