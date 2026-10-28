//
// ImTwo.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImTwoInput : MLFeatureProvider {

    /// left as color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
    public var left_: CVPixelBuffer

    /// right as color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
    public var right_: CVPixelBuffer

    public var featureNames: Set<String> { ["left", "right"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "left" {
            return MLFeatureValue(pixelBuffer: left_)
        }
        if featureName == "right" {
            return MLFeatureValue(pixelBuffer: right_)
        }
        return nil
    }

    public init(left_: CVPixelBuffer, right_: CVPixelBuffer) {
        self.left_ = left_
        self.right_ = right_
    }

    public convenience init(left_With left_: CGImage, right_With right_: CGImage) throws {
        self.init(left_: try MLFeatureValue(cgImage: left_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, right_: try MLFeatureValue(cgImage: right_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!)
    }

    public convenience init(left_At left_: URL, right_At right_: URL) throws {
        self.init(left_: try MLFeatureValue(imageAt: left_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, right_: try MLFeatureValue(imageAt: right_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!)
    }

    public func setLeft_(with left_: CGImage) throws  {
        self.left_ = try MLFeatureValue(cgImage: left_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setRight_(with right_: CGImage) throws  {
        self.right_ = try MLFeatureValue(cgImage: right_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setLeft_(with left_: URL) throws  {
        self.left_ = try MLFeatureValue(imageAt: left_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setRight_(with right_: URL) throws  {
        self.right_ = try MLFeatureValue(imageAt: right_, pixelsWide: 8, pixelsHigh: 8, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImTwoOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reshape_0 as 1 element vector of floats
    public var reshape_0: MLMultiArray {
        provider.featureValue(for: "reshape_0")!.multiArrayValue!
    }

    /// reshape_0 as 1 element vector of floats
    public var reshape_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reshape_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reshape_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reshape_0" : MLFeatureValue(multiArray: reshape_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImTwo {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ImTwo", withExtension:"mlmodelc")!
    }

    /**
        Construct ImTwo instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ImTwo.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ImTwo.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct ImTwo instance with explicit path to mlmodelc file
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
        Construct ImTwo instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImTwo, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ImTwo instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImTwo {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ImTwo instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImTwo, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ImTwo(model: model)))
            }
        }
    }

    /**
        Construct ImTwo instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImTwo {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ImTwo(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImTwoInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImTwoOutput
    */
    public func prediction(input: ImTwoInput) throws -> ImTwoOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImTwoInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImTwoOutput
    */
    public func prediction(input: ImTwoInput, options: MLPredictionOptions) throws -> ImTwoOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ImTwoOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImTwoInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImTwoOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ImTwoInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ImTwoOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ImTwoOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - left_: color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high
            - right_: color (kCVPixelFormatType_32BGRA) image buffer, 8 pixels wide by 8 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImTwoOutput
    */
    public func prediction(left_: CVPixelBuffer, right_: CVPixelBuffer) throws -> ImTwoOutput {
        let input_ = ImTwoInput(left_: left_, right_: right_)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ImTwoInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ImTwoOutput]
    */
    public func predictions(inputs: [ImTwoInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ImTwoOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ImTwoOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ImTwoOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
