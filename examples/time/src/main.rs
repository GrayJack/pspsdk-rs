#![no_main]
#![no_std]

use core::mem::MaybeUninit;

use pspsdk::sys::SceResult;

pspsdk::module!("sample_time", 1, 1);

fn psp_main() -> SceResult<()> {
    pspsdk::enable_home_button();

    unsafe {
        let mut tick = 0;
        pspsdk::sys::time::sceRtcGetCurrentTick(&mut tick)?;

        // Convert the tick to an instance of `ScePspDateTime`
        let mut date = MaybeUninit::uninit();
        pspsdk::sys::time::sceRtcSetTick(date.as_mut_ptr(), &tick)?;
        let date = date.assume_init();

        pspsdk::dprintln!(
            "Current time is {:02}:{:02}:{:02} UTC",
            date.hour,
            date.minutes,
            date.seconds
        );
    }

    SceResult::OK
}
