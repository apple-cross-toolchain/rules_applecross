//
//  MyC4+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias MyC4CoreDataPropertiesSet = NSSet

extension MyC4 {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<MyC4> {
        return NSFetchRequest<MyC4>(entityName: "C4")
    }

    @NSManaged nonisolated public var a: String?

}

extension MyC4 : Identifiable {

}
