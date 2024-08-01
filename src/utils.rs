//arc_mutex_var
#[macro_export]
macro_rules! set_arc_mutex_var {
    ($i:ident,$v:expr) => {
        let vcl = $i.clone();
        let mut vcl_guard = vcl.lock().unwrap();
        *vcl_guard = $v;
        drop(vcl_guard);
        drop(vcl);
    };
}
#[macro_export]
macro_rules! get_arc_mutex_var {
    ($i:ident) => {
        {let vcl = $i.clone();
        let vcl_guard = vcl.lock().unwrap();
        let ret = *vcl_guard;
        drop(vcl_guard);
        drop(vcl);
        ret}
    }
}
#[macro_export]
macro_rules! push_arc_mutex_var {
    ($i:ident,$v:expr) => {
        let vcl = $i.clone();
        let mut vcl_guard = vcl.lock().unwrap();
        vcl_guard.push($v);
        drop(vcl_guard);
        drop(vcl);
    };
}

#[macro_export]
macro_rules! coddition_arc_mutex_var {
    ($i:ident,$v:expr,$c:expr) => {
        let vcl = $i.clone();
        let vcl_guard = vcl.lock().unwrap();
        if *vcl_guard == $v {
            drop(vcl_guard);
            drop(vcl);
            $c;
        }
        drop(vcl_guard);
        drop(vcl);
    };
}
