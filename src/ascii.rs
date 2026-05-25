// ASCII Art Logos for GPU vendors

pub const NVIDIA_SHORT: &str = r#"
$C1               'cccccccccccccccccccccccccc   
$C1               ;oooooooooooooooooooooooool   
$C1           .:::.     .oooooooooooooooooool   
$C1      .:cll;   ,c:::.     cooooooooooooool   
$C1   ,clo'      ;.   oolc:     ooooooooooool   
$C1.cloo    ;cclo .      .olc.    coooooooool   
$C1oooo   :lo,    ;ll;    looc    :oooooooool   
$C1 oooc   ool.   ;oooc;clol    :looooooooool   
$C1  :ooc   ,ol;  ;oooooo.   .cloo;     loool   
$C1    ool;   .olc.       ,:lool        .loool   
$C1      ool:.    ,::::ccloo.        :clooool   
$C1         oolc::.            ':cclooooooool   
$C1               ;oooooooooooooooooooooooool   
$C1                                             
$C1                                             
$C1######.  ##   ##  ##  ######   ##    ###     
$C1##   ##  ##   ##  ##  ##   ##  ##   #: :#    
$C1##   ##   ## ##   ##  ##   ##  ##  #######   
$C1##   ##    ###    ##  ######   ## ##     ##  
"#;

pub const NVIDIA_LONG: &str = r#"
$C1                  MMMMMMMMMMMMMMMMMMMMMMMMMMMMMM  
$C1                  MMMMMMMMMMMMMMMMMMMMMMMMMMMMMM  
$C1                .::   'MMMMMMMMMMMMMMMMMMMMMMMMM  
$C1           ccllooo;:;.       ;MMMMMMMMMMMMMMMMMM  
$C1       cloc       :ooollcc:     :MMMMMMMMMMMMMMM  
$C1    cloc      :ccl;      lolc,     ;MMMMMMMMMMMM  
$C1.cloo:    :clo    ;c:      .ool;     MMMMMMMMMMM  
$C1  ooo:    ooo     :ool,  .cloo.    ;lMMMMMMMMMMM  
$C1   ooo:    ooc    :ooooccooo.    :MMMM  lMMMMMMM  
$C1     ooc.   ool:  :oooooo'    ,cloo.        MMMM  
$C1      ool:.    olc:       .:cloo.          :MMMM  
$C1         olc,     ;:::cccloo.          :MMMMMMMM  
$C1            olcc::;              ,:ccloMMMMMMMMM  
$C1                  :......oMMMMMMMMMMMMMMMMMMMMMM  
$C1                  :lllMMMMMMMMMMMMMMMMMMMMMMMMMM  
"#;

pub const AMD_SHORT: &str = r#"
$C1          '###############             
$C1             ,#############            
$C1                      .####            
$C1              #.      .####            
$C1            :##.      .####            
$C1           :###.      .####            
$C1           #########.   :##            
$C1           #######.       ;            
$C1    ###     ###      ###   #######     
$C1   ## ##    #####  #####   ##     ##   
$C1  ##   ##   ### #### ###   ##      ##  
$C1 #########  ###  ##  ###   ##      ##  
$C1##       ## ###      ###   ##     ##   
$C1##       ## ###      ###   #######     
"#;

pub const AMD_LONG: &str = r#"
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
$C1     @@@@      @@@       @@@   @@@@@@@@      $C2  ############   
$C1    @@@@@@     @@@@@   @@@@@   @@@    @@@    $C2    ##########   
$C1   @@@  @@@    @@@@@@@@@@@@@   @@@      @@   $C2   #     #####   
$C1  @@@    @@@   @@@  @@@  @@@   @@@      @@   $C2 ###     #####   
$C1 @@@@@@@@@@@@  @@@       @@@   @@@    @@@    $C2#########  ###   
$C1 @@@      @@@  @@@       @@@   @@@@@@@@@     $C2########    ##   
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
$C1                                                             
"#;

pub const INTEL_SHORT: &str = r#"
$C1                   .#################.          
$C1              .####                   ####.     
$C1          .##                             ###   
$C1       ##                          :##     ###  
$C1    #                ##            :##      ##  
$C1  ##   ##  ######.   ####  ######  :##      ##  
$C1 ##    ##  ##:  ##:  ##   ##   ### :##     ###  
$C1##     ##  ##:  ##:  ##  :######## :##    ##    
$C1##     ##  ##:  ##:  ##   ##.   .  :## ####     
$C1##      #  ##:  ##:  ####  #####:   ##          
$C1 ##                                             
$C1  ###.                         ..o####.         
$C1   ######oo...         ..oo#######              
$C1          o###############o                     
"#;

pub const INTEL_LONG: &str = r#"
$C1                               ###############@               
$C1                       ######@                ######@         
$C1                  ###@                              ###@      
$C1              ##@                                     ###@    
$C1         ##@                                             ##@  
$C1         ##@                                             ##@  
$C1      @                    ##@                ##@        ##@  
$C1    #@   ##@   ########@   #####@   #####@    ##@        ##@  
$C1   #@    ##@   ##@    ##@  ##@    ###@  ###@  ##@        ##@  
$C1  #@     ##@   ##@    ##@  ##@    ##@    ##@  ##@       ##@   
$C1 #@      ##@   ##@    ##@  ##@    #########@  ##@     ###@    
$C1 #@      ##@   ##@    ##@  ##@    ##@         ##@   ####@     
$C1 #@       #@   ##@    ##@   ####@  ########@   #@  ##@        
$C1 ##@                                                          
$C1  ##@                                                         
$C1  ###@                                        ###@            
$C1    ####@                               #########@            
$C1      #########@               ###############@               
$C1          ##############################@                     
"#;

pub struct Logo {
    pub art: &'static str,
    pub width: u32,
    pub height: u32,
    pub replace_blocks: bool,
}

pub fn get_logo_nvidia(long: bool) -> Logo {
    if long {
        Logo {
            art: NVIDIA_LONG,
            width: 50,
            height: 15,
            replace_blocks: false,
        }
    } else {
        Logo {
            art: NVIDIA_SHORT,
            width: 45,
            height: 19,
            replace_blocks: false,
        }
    }
}

pub fn get_logo_amd(long: bool) -> Logo {
    if long {
        Logo {
            art: AMD_LONG,
            width: 62,
            height: 19,
            replace_blocks: true,
        }
    } else {
        Logo {
            art: AMD_SHORT,
            width: 39,
            height: 14,
            replace_blocks: false,
        }
    }
}

pub fn get_logo_intel(long: bool) -> Logo {
    if long {
        Logo {
            art: INTEL_LONG,
            width: 62,
            height: 19,
            replace_blocks: true,
        }
    } else {
        Logo {
            art: INTEL_SHORT,
            width: 48,
            height: 14,
            replace_blocks: false,
        }
    }
}
