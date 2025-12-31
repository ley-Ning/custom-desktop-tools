//
//  Item.swift
//  my-utools
//
//  Created by wen on 2025/12/31.
//

import Foundation
import SwiftData

@Model
final class Item {
    var timestamp: Date
    
    init(timestamp: Date) {
        self.timestamp = timestamp
    }
}
