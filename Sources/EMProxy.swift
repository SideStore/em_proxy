//
//  EMProxy.swift
//  em_proxy
//
//  Created by Magesh K on 8/8/26.
//  Copyright © 2026 SideStore. All rights reserved.
//

import Foundation
import NativeBridge

public enum EMProxy {
    public static func start(bindAddress: String) {
        let host = NSString(string: bindAddress)
        let hostPointer = UnsafeMutablePointer<CChar>(mutating: host.utf8String)
        _ = start_emotional_damage(hostPointer)
    }

    public static func stop() {
        stop_emotional_damage()
    }
}
