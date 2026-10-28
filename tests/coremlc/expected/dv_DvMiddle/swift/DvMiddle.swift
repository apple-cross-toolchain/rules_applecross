//
// DvMiddle.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class DvMiddleInput : MLFeatureProvider {

    /// a as 1 by 2 matrix of floats
    public var a: MLMultiArray

    /// b as optional 1 by 2 matrix of floats
    public var b: MLMultiArray? = nil

    /// c as 1 by 2 matrix of floats
    public var c: MLMultiArray

    public var featureNames: Set<String> { ["a", "b", "c"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "a" {
            return MLFeatureValue(multiArray: a)
        }
        if featureName == "b" {
            return b == nil ? nil : MLFeatureValue(multiArray: b!)
        }
        if featureName == "c" {
            return MLFeatureValue(multiArray: c)
        }
        return nil
    }

    public init(a: MLMultiArray, c: MLMultiArray, b: MLMultiArray? = nil) {
        self.a = a
        self.c = c
        self.b = b
    }

    public convenience init(a: MLShapedArray<Float>, c: MLShapedArray<Float>, b: MLShapedArray<Float>? = nil) {
        self.init(a: MLMultiArray(a), c: MLMultiArray(c), b: b != nil ? MLMultiArray(b!) : nil)
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class DvMiddleOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// add_1 as 1 by 2 matrix of floats
    public var add_1: MLMultiArray {
        provider.featureValue(for: "add_1")!.multiArrayValue!
    }

    /// add_1 as 1 by 2 matrix of floats
    public var add_1ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(add_1)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(add_1: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["add_1" : MLFeatureValue(multiArray: add_1)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class DvMiddle {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "DvMiddle", withExtension:"mlmodelc")!
    }

    /**
        Construct DvMiddle instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of DvMiddle.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `DvMiddle.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct DvMiddle instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct DvMiddle instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<DvMiddle, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct DvMiddle instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> DvMiddle {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct DvMiddle instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<DvMiddle, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(DvMiddle(model: model)))
            }
        }
    }

    /**
        Construct DvMiddle instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> DvMiddle {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return DvMiddle(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DvMiddleInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvMiddleOutput
    */
    public func prediction(input: DvMiddleInput) throws -> DvMiddleOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DvMiddleInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvMiddleOutput
    */
    public func prediction(input: DvMiddleInput, options: MLPredictionOptions) throws -> DvMiddleOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return DvMiddleOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DvMiddleInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvMiddleOutput
    */
    public func prediction(input: DvMiddleInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> DvMiddleOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return DvMiddleOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: 1 by 2 matrix of floats
            - c: 1 by 2 matrix of floats
            - b: optional 1 by 2 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvMiddleOutput
    */
    public func prediction(a: MLMultiArray, c: MLMultiArray, b: MLMultiArray?) throws -> DvMiddleOutput {
        let input_ = DvMiddleInput(a: a, c: c, b: b)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: 1 by 2 matrix of floats
            - c: 1 by 2 matrix of floats
            - b: optional 1 by 2 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvMiddleOutput
    */

    public func prediction(a: MLShapedArray<Float>, c: MLShapedArray<Float>, b: MLShapedArray<Float>?) throws -> DvMiddleOutput {
        let input_ = DvMiddleInput(a: a, c: c, b: b)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [DvMiddleInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [DvMiddleOutput]
    */
    public func predictions(inputs: [DvMiddleInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [DvMiddleOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [DvMiddleOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  DvMiddleOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
