macro_rules! remove_overriden {
    (@remove_requires $rem_from:expr, $a:ident.$ov:ident) => {
        if let Some(ora) = $a.$ov() {
            for i in (0 .. $rem_from.len()).rev() {
                let should_remove = ora.iter().any(|&(_, ref name)| name == &$rem_from[i]);
                if should_remove { $rem_from.swap_remove(i); }
            }
        }
    };
    (@remove $rem_from:expr, $a:ident.$ov:ident) => {
        if let Some(ora) = $a.$ov() {
            vec_remove_all!($rem_from, ora.iter());
        }
    };
    (@arg $_self:ident, $arg:ident) => {
        remove_overriden!(@remove_requires $_self.required, $arg.requires);
        remove_overriden!(@remove $_self.blacklist, $arg.blacklist);
        remove_overriden!(@remove $_self.overrides, $arg.overrides);
    };
    ($_self:ident, $name:expr) => {
        debugln!("remove_overriden!;");
        if let Some(o) = $_self.opts.iter() .find(|o| o.b.name == *$name) {
            remove_overriden!(@arg $_self, o);
        } else if let Some(f) = $_self.flags.iter() .find(|f| f.b.name == *$name) {
            remove_overriden!(@arg $_self, f);
        } else {
            let p = $_self.positionals.values()
                                      .find(|p| p.b.name == *$name)
                                      .expect(INTERNAL_ERROR_MSG);
            remove_overriden!(@arg $_self, p);
        }
    };
}

macro_rules! _handle_group_reqs{
    ($me:ident, $arg:ident) => ({
        debugln!("_handle_group_reqs!;");
        for grp in &$me.groups {
            let found = if grp.args.contains(&$arg.name()) {
                if let Some(ref reqs) = grp.requires {
                    debugln!("_handle_group_reqs!: Adding {:?} to the required list", reqs);
                    $me.required.extend(reqs);
                }
                if let Some(ref bl) = grp.conflicts {
                    $me.blacklist.extend(bl);
                }
                true // What if arg is in more than one group with different reqs?
            } else {
                false
            };
            debugln!("_handle_group_reqs!:iter: grp={}, found={:?}", grp.name, found);
            if found {
                for i in (0 .. $me.required.len()).rev() {
                    let should_remove = grp.args.contains(&$me.required[i]);
                    if should_remove { $me.required.swap_remove(i); }
                }
                debugln!("_handle_group_reqs!:iter: Adding args from group to blacklist...{:?}", grp.args);
                if !grp.multiple {
                    $me.blacklist.extend(&grp.args);
                    debugln!("_handle_group_reqs!: removing {:?} from blacklist", $arg.name());
                    for i in (0 .. $me.blacklist.len()).rev() {
                        let should_remove = $me.blacklist[i] == $arg.name();
                        if should_remove { $me.blacklist.swap_remove(i); }
                    }
                }
            }
        }
    })
}

macro_rules! parse_positional {
    (
        $_self:ident,
        $p:ident,
        $arg_os:ident,
        $pos_counter:ident,
        $matcher:ident
    ) => {
        debugln!("parse_positional!;");

        if !$_self.is_set(AS::TrailingValues) &&
           ($_self.is_set(AS::TrailingVarArg) &&
            $pos_counter == $_self.positionals.len()) {
            $_self.settings.set(AS::TrailingValues);
        }
        let _ = $_self.add_val_to_arg($p, &$arg_os, $matcher)?;

        $matcher.inc_occurrence_of($p.b.name);
        let _ = $_self.groups_for_arg($p.b.name)
                      .and_then(|vec| Some($matcher.inc_occurrences_of(&*vec)));
        if $_self.cache.map_or(true, |name| name != $p.b.name) {
            $_self.cache = Some($p.b.name);
        }

        $_self.settings.set(AS::ValidArgFound);
        // Only increment the positional counter if it doesn't allow multiples
        if !$p.b.settings.is_set(ArgSettings::Multiple) {
            $pos_counter += 1;
        }
    };
}
