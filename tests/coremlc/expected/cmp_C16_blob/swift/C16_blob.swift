//
// C16_blob.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class C16_blobInput : MLFeatureProvider {

    /// x as 4 by 4 matrix of floats
    public var x: MLMultiArray

    public var featureNames: Set<String> { ["x"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        return nil
    }

    public init(x: MLMultiArray) {
        self.x = x
    }

    public convenience init(x: MLShapedArray<Float>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class C16_blobOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// add_0 as 4 by 4 matrix of floats
    public var add_0: MLMultiArray {
        provider.featureValue(for: "add_0")!.multiArrayValue!
    }

    /// add_0 as 4 by 4 matrix of floats
    public var add_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(add_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(add_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["add_0" : MLFeatureValue(multiArray: add_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class C16_blob {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "C16_blob", withExtension:"mlmodelc")!
    }

    /**
        Construct C16_blob instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of C16_blob.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `C16_blob.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct C16_blob instance with explicit path to mlmodelc file
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
        Construct C16_blob instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<C16_blob, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct C16_blob instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> C16_blob {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct C16_blob instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<C16_blob, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(C16_blob(model: model)))
            }
        }
    }

    /**
        Construct C16_blob instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> C16_blob {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return C16_blob(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as C16_blobInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as C16_blobOutput
    */
    public func prediction(input: C16_blobInput) throws -> C16_blobOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as C16_blobInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as C16_blobOutput
    */
    public func prediction(input: C16_blobInput, options: MLPredictionOptions) throws -> C16_blobOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return C16_blobOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as C16_blobInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as C16_blobOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: C16_blobInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> C16_blobOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return C16_blobOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 4 by 4 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as C16_blobOutput
    */
    public func prediction(x: MLMultiArray) throws -> C16_blobOutput {
        let input_ = C16_blobInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 4 by 4 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as C16_blobOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> C16_blobOutput {
        let input_ = C16_blobInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [C16_blobInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [C16_blobOutput]
    */
    public func predictions(inputs: [C16_blobInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [C16_blobOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [C16_blobOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  C16_blobOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
