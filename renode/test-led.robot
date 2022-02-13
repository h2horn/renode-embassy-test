*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Test Cases ***
Test Uart Led Toggle
    Execute Command         mach create
    Execute Command         machine LoadPlatformDescription @${PWD_PATH}/nrf52840-dongle.repl
    Execute Command         sysbus LoadELF @${PWD_PATH}/../firmware/target/thumbv7em-none-eabi/debug/frenode

    Create Terminal Tester  sysbus.uart0
    Execute Command         emulation CreateLEDTester "lt" sysbus.gpio0.led

    Start Emulation

    Wait For Line On Uart   Hello World!
    Execute Command         lt AssertState False 0
    Write Char On Uart      1
    Write Char On Uart      \n
    Test If Uart Is Idle    1
    Execute Command         lt AssertState True 0
    Write Char On Uart      0
    Write Char On Uart      \n
    Test If Uart Is Idle    1
    Execute Command         lt AssertState False 0
