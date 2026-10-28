//
// I17_g16_range_wfix.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class I17_g16_range_wfixInput : MLFeatureProvider {

    /// img as grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 6 pixels wide by 4 pixels high
    public var img: CVPixelBuffer

    public var featureNames: Set<String> { ["img"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "img" {
            return MLFeatureValue(pixelBuffer: img)
        }
        return nil
    }

    public init(img: CVPixelBuffer) {
        self.img = img
    }

    public convenience init(imgWith img: CGImage) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!)
    }

    public convenience init(imgAt img: URL) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!)
    }

    public func setImg(with img: CGImage) throws  {
        self.img = try MLFeatureValue(cgImage: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

    public func setImg(with img: URL) throws  {
        self.img = try MLFeatureValue(imageAt: img, pixelsWide: 6, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_OneComponent16Half, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class I17_g16_range_wfixOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
    public var reduce_mean_0: MLMultiArray {
        provider.featureValue(for: "reduce_mean_0")!.multiArrayValue!
    }

    /// reduce_mean_0 as 1 by 1 matrix of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #else
    @available(macOS 15.0, *)
    #endif
    public var reduce_mean_0ShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(reduce_mean_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reduce_mean_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reduce_mean_0" : MLFeatureValue(multiArray: reduce_mean_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class I17_g16_range_wfix {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "I17_g16_range_wfix", withExtension:"mlmodelc")!
    }

    /**
        Construct I17_g16_range_wfix instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of I17_g16_range_wfix.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `I17_g16_range_wfix.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct I17_g16_range_wfix instance with explicit path to mlmodelc file
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
        Construct I17_g16_range_wfix instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<I17_g16_range_wfix, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct I17_g16_range_wfix instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> I17_g16_range_wfix {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct I17_g16_range_wfix instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<I17_g16_range_wfix, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(I17_g16_range_wfix(model: model)))
            }
        }
    }

    /**
        Construct I17_g16_range_wfix instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> I17_g16_range_wfix {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return I17_g16_range_wfix(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as I17_g16_range_wfixInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as I17_g16_range_wfixOutput
    */
    public func prediction(input: I17_g16_range_wfixInput) throws -> I17_g16_range_wfixOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as I17_g16_range_wfixInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as I17_g16_range_wfixOutput
    */
    public func prediction(input: I17_g16_range_wfixInput, options: MLPredictionOptions) throws -> I17_g16_range_wfixOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return I17_g16_range_wfixOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as I17_g16_range_wfixInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as I17_g16_range_wfixOutput
    */
    public func prediction(input: I17_g16_range_wfixInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> I17_g16_range_wfixOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return I17_g16_range_wfixOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: grayscale (kCVPixelFormatType_OneComponent16Half) image buffer, 6 pixels wide by 4 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as I17_g16_range_wfixOutput
    */
    public func prediction(img: CVPixelBuffer) throws -> I17_g16_range_wfixOutput {
        let input_ = I17_g16_range_wfixInput(img: img)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [I17_g16_range_wfixInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [I17_g16_range_wfixOutput]
    */
    public func predictions(inputs: [I17_g16_range_wfixInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [I17_g16_range_wfixOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [I17_g16_range_wfixOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  I17_g16_range_wfixOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
