//
// Op_pixel_unshuffle_16.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class Op_pixel_unshuffle_16Input : MLFeatureProvider {

    /// x as 1 × 1 × 4 × 4 4-dimensional array of floats
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
public class Op_pixel_unshuffle_16Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// o as 1 × 4 × 2 × 2 4-dimensional array of floats
    public var o: MLMultiArray {
        provider.featureValue(for: "o")!.multiArrayValue!
    }

    /// o as 1 × 4 × 2 × 2 4-dimensional array of floats
    public var oShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(o)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(o: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["o" : MLFeatureValue(multiArray: o)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class Op_pixel_unshuffle_16 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Op_pixel_unshuffle_16", withExtension:"mlmodelc")!
    }

    /**
        Construct Op_pixel_unshuffle_16 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Op_pixel_unshuffle_16.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Op_pixel_unshuffle_16.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Op_pixel_unshuffle_16 instance with explicit path to mlmodelc file
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
        Construct Op_pixel_unshuffle_16 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Op_pixel_unshuffle_16, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Op_pixel_unshuffle_16 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Op_pixel_unshuffle_16 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Op_pixel_unshuffle_16 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Op_pixel_unshuffle_16, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Op_pixel_unshuffle_16(model: model)))
            }
        }
    }

    /**
        Construct Op_pixel_unshuffle_16 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Op_pixel_unshuffle_16 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Op_pixel_unshuffle_16(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Op_pixel_unshuffle_16Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Op_pixel_unshuffle_16Output
    */
    public func prediction(input: Op_pixel_unshuffle_16Input) throws -> Op_pixel_unshuffle_16Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Op_pixel_unshuffle_16Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Op_pixel_unshuffle_16Output
    */
    public func prediction(input: Op_pixel_unshuffle_16Input, options: MLPredictionOptions) throws -> Op_pixel_unshuffle_16Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return Op_pixel_unshuffle_16Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Op_pixel_unshuffle_16Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Op_pixel_unshuffle_16Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: Op_pixel_unshuffle_16Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Op_pixel_unshuffle_16Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Op_pixel_unshuffle_16Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 1 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Op_pixel_unshuffle_16Output
    */
    public func prediction(x: MLMultiArray) throws -> Op_pixel_unshuffle_16Output {
        let input_ = Op_pixel_unshuffle_16Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 × 1 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Op_pixel_unshuffle_16Output
    */

    public func prediction(x: MLShapedArray<Float>) throws -> Op_pixel_unshuffle_16Output {
        let input_ = Op_pixel_unshuffle_16Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Op_pixel_unshuffle_16Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Op_pixel_unshuffle_16Output]
    */
    public func predictions(inputs: [Op_pixel_unshuffle_16Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Op_pixel_unshuffle_16Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Op_pixel_unshuffle_16Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Op_pixel_unshuffle_16Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
