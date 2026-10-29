//
// mlp_extra_weightfile.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class mlp_extra_weightfileInput : MLFeatureProvider {

    /// x as 1 by 64 matrix of floats
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
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class mlp_extra_weightfileOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// out as 1 by 32 matrix of floats
    public var out_: MLMultiArray {
        provider.featureValue(for: "out")!.multiArrayValue!
    }

    /// out as 1 by 32 matrix of floats
    public var out_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(out_)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(out_: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["out" : MLFeatureValue(multiArray: out_)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class mlp_extra_weightfile {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "mlp_extra_weightfile", withExtension:"mlmodelc")!
    }

    /**
        Construct mlp_extra_weightfile instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of mlp_extra_weightfile.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `mlp_extra_weightfile.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct mlp_extra_weightfile instance with explicit path to mlmodelc file
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
        Construct mlp_extra_weightfile instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<mlp_extra_weightfile, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct mlp_extra_weightfile instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> mlp_extra_weightfile {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct mlp_extra_weightfile instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<mlp_extra_weightfile, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(mlp_extra_weightfile(model: model)))
            }
        }
    }

    /**
        Construct mlp_extra_weightfile instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> mlp_extra_weightfile {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return mlp_extra_weightfile(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as mlp_extra_weightfileInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as mlp_extra_weightfileOutput
    */
    public func prediction(input: mlp_extra_weightfileInput) throws -> mlp_extra_weightfileOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as mlp_extra_weightfileInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as mlp_extra_weightfileOutput
    */
    public func prediction(input: mlp_extra_weightfileInput, options: MLPredictionOptions) throws -> mlp_extra_weightfileOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return mlp_extra_weightfileOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as mlp_extra_weightfileInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as mlp_extra_weightfileOutput
    */
    public func prediction(input: mlp_extra_weightfileInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> mlp_extra_weightfileOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return mlp_extra_weightfileOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 64 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as mlp_extra_weightfileOutput
    */
    public func prediction(x: MLMultiArray) throws -> mlp_extra_weightfileOutput {
        let input_ = mlp_extra_weightfileInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 64 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as mlp_extra_weightfileOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> mlp_extra_weightfileOutput {
        let input_ = mlp_extra_weightfileInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [mlp_extra_weightfileInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [mlp_extra_weightfileOutput]
    */
    public func predictions(inputs: [mlp_extra_weightfileInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [mlp_extra_weightfileOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [mlp_extra_weightfileOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  mlp_extra_weightfileOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
